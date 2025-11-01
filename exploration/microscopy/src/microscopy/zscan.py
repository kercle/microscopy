import io

from PIL import Image


class ZScan:
    def __init__(
        self,
        connection: object,
        relative_start_pos: int,
        relative_stop_pos: int,
        steps_between_layers: int,
        frame_count: int,
        uuid: str,
    ):
        self.connection = connection

        self.relative_start_pos = relative_start_pos
        self.relative_stop_pos = relative_stop_pos
        self.steps_between_layers = steps_between_layers
        self.frame_count = frame_count
        self.uuid = uuid

    def thumbnail(self, frame: int, size: int = 128) -> bytes:
        endpoint = f"/z-scan/thumbnail/{self.uuid}/{frame}/{size}"
        data = self.connection.get_bytes(endpoint)
        return Image.open(io.BytesIO(data))


class ZScanRepository:
    def __init__(self, connection: object):
        self.connection = connection

    def _build_endpoint(self, endpoint: str) -> str:
        return f"/z-scan/{endpoint.lstrip('/')}"

    def ls(self) -> list:
        endpoint = self._build_endpoint("list")
        return self.connection.get_json(endpoint)

    def select(self, idx: int) -> ZScan:
        all_scans = self.ls()
        selected_scan = all_scans[idx]

        return ZScan(
            self.connection,
            selected_scan["relative_start_pos"],
            selected_scan["relative_stop_pos"],
            selected_scan["steps_between_layers"],
            selected_scan["frame_count"],
            selected_scan["uuid"],
        )
