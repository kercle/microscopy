import os
import tempfile
from enum import Enum

import numpy as np
import scipy
import cv2

from microscopy import kernels
from microscopy.utils import passthrough_iter


class FocusScoreAlgorithm(Enum):
    TENENGRAD = 1


class FocusStacking:
    def __init__(
        self,
        data: np.ndarray | np.memmap,
    ):
        self.image_data = data
        self.focus_data = None
        self.memory_mapped_files = []

    def _make_tempfile(self, prefix: str, suffix: str, **kwargs) -> np.memmap:
        temp_file = tempfile.NamedTemporaryFile(
            prefix=f"microscopy-memmapped-{prefix}",
            suffix=suffix,
            **kwargs,
        )
        return temp_file

    def _make_tempfile_name(self, prefix: str, suffix: str) -> str:
        temp_file = self._make_tempfile(
            prefix=prefix,
            suffix=suffix,
            delete=False,
            delete_on_close=False,
        )
        file_name = temp_file.name
        self.memory_mapped_files.append(file_name)
        temp_file.close()
        return file_name

    def from_zscan_thumbnails(zscan: object, size: int) -> object:
        thumbnails = zscan.thumbails_as_array(size)
        return FocusStacking(thumbnails)

    def from_zscan_frames(zscan: object, progress_iter: iter = passthrough_iter) -> object:
        obj = FocusStacking(np.empty(0))
        file_name = obj._make_tempfile_name(prefix="zscan_frames_", suffix=".dat")

        first_frame = np.array(zscan.frame(0))
        data = np.memmap(
            file_name,
            dtype=first_frame.dtype,
            mode="w+",
            shape=(zscan.frame_count, *first_frame.shape),
        )

        data[0] = first_frame
        for i in progress_iter(range(1, zscan.frame_count)):
            data[i] = np.array(zscan.frame(i))

        obj.image_data = data
        return obj

    def _focus_score_tenengrad(self, kernel, progress_iter: iter) -> np.ndarray:
        focus_map = np.memmap(
            self._make_tempfile_name(prefix="focus_map_tenengrad_", suffix=".dat"),
            dtype=np.float32,
            mode="w+",
            shape=self.image_data.shape[:3],
        )

        for i in progress_iter(range(self.image_data.shape[0])):
            gray = cv2.cvtColor(self.image_data[i], cv2.COLOR_RGB2GRAY)

            gradient_x = cv2.Sobel(gray, cv2.CV_64F, 1, 0, ksize=3)
            gradient_y = cv2.Sobel(gray, cv2.CV_64F, 0, 1, ksize=3)

            tenengrad = np.sqrt(gradient_x**2 + gradient_y**2)

            focus_map[i, :, :] = scipy.signal.convolve2d(tenengrad, kernel, mode="same")

        return focus_map

    def focus_score(
        self,
        algorithm: FocusScoreAlgorithm = FocusScoreAlgorithm.TENENGRAD,
        progress_iter: iter = passthrough_iter,
    ) -> np.ndarray:
        if self.focus_data is not None:
            return self.focus_data

        kernel = kernels.gaussian(l=20, sig=5)

        if algorithm == FocusScoreAlgorithm.TENENGRAD:
            self.focus_data = self._focus_score_tenengrad(kernel, progress_iter)
        else:
            raise ValueError(f"Unsupported sharpness algorithm: {algorithm}")
        
        return self.focus_data

    def condense(
        self,
        algorithm: FocusScoreAlgorithm = FocusScoreAlgorithm.TENENGRAD,
        progress_iter: iter = passthrough_iter,
    ) -> np.ndarray:
        scores = self.focus_score(algorithm, progress_iter)

        idx = scores.argmax(axis=0)
        rows, cols = np.indices(idx.shape)

        if self.image_data.ndim == 4:
            mask_shape = self.image_data.shape[:3]

        tempfile = self._make_tempfile(
            prefix="condense_mask_",
            suffix=".dat",
        )
        mask = np.memmap(
            tempfile.name,
            dtype=np.uint8,
            mode="w+",
            shape=self.image_data.shape,
        )

        if self.image_data.ndim == 4:
            mask[idx, rows, cols, :] = 1
        elif self.image_data.ndim == 3:
            mask[idx, rows, cols] = 1
        else:
            raise ValueError("Image data must be 3D or 4D array.")

        return np.sum(self.image_data * mask, axis=0).astype(self.image_data.dtype)

    def cleanup(self):
        for file_name in self.memory_mapped_files:
            try:
                os.remove(file_name)
            except OSError:
                pass

        self.memory_mapped_files = []

    def __del__(self):
        self.cleanup()
