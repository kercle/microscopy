from enum import Enum

import numpy as np
import scipy
import cv2

from microscopy import kernels


class FocusScoreAlgorithm(Enum):
    TENENGRAD = 1


class FocusStacking:
    def __init__(
        self,
        data: np.ndarray,
    ):
        self.data = data

    def from_zscan(zscan: object, size: int) -> object:
        thumbnails = zscan.thumbails_as_array(size)
        return FocusStacking(thumbnails)

    def _focus_score_tenengrad(self, kernel) -> np.ndarray:
        sharpness_map = np.empty(self.data.shape[:-1])

        for i in range(self.data.shape[0]):
            gray = cv2.cvtColor(self.data[i], cv2.COLOR_RGB2GRAY)

            gradient_x = cv2.Sobel(gray, cv2.CV_64F, 1, 0, ksize=3)
            gradient_y = cv2.Sobel(gray, cv2.CV_64F, 0, 1, ksize=3)

            tenengrad = np.sqrt(gradient_x**2 + gradient_y**2)

            sharpness_map[i, :, :] = scipy.signal.convolve2d(
                tenengrad, kernel, mode="same"
            )

        return sharpness_map

    def focus_score(
        self, algorithm: FocusScoreAlgorithm = FocusScoreAlgorithm.TENENGRAD
    ) -> np.ndarray:
        kernel = kernels.gaussian(l=20, sig=5)

        if algorithm == FocusScoreAlgorithm.TENENGRAD:
            return self._focus_score_tenengrad(kernel)
        else:
            raise ValueError(f"Unsupported sharpness algorithm: {algorithm}")

    def condense(self, algorithm: FocusScoreAlgorithm = FocusScoreAlgorithm.TENENGRAD) -> np.ndarray:
        scores = self.focus_score(algorithm)

        idx = scores.argmax(axis=0)
        rows, cols = np.indices(idx.shape)

        mask = np.zeros(self.data.shape[:3])
        mask[idx, rows, cols] = 1

        if self.data.ndim == 4:
            mask = np.repeat(mask[:, :, :, np.newaxis], self.data.shape[3], axis=3)

        return np.sum(self.data * mask, axis=0).astype(self.data.dtype)
