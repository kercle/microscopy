import requests

API_PATH_BASE = "/api"


class Http:
    def __init__(self, host: str, port: int):
        self.host = host
        self.port = port

    def build_url_raw(self, full_endpoint: str) -> str:
        return f"http://{self.host}:{self.port}/{full_endpoint.lstrip('/')}"

    def _get(self, endpoint: str) -> object:
        full_endpoint = f"{API_PATH_BASE}/{endpoint.lstrip('/')}"
        url = self.build_url_raw(full_endpoint)

        response = requests.get(url)
        response.raise_for_status()
        return response

    def get_json(self, endpoint: str) -> str:
        response = self._get(endpoint)
        return response.json()

    def get_bytes(self, endpoint: str) -> bytes:
        response = self._get(endpoint)
        return response.content
