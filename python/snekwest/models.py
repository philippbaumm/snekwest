from snekwest._bindings import Response as RustResponse


class Response:
    def __init__(self, rust_response: RustResponse) -> None:
        self._rust_response = rust_response

        self.status_code: int = rust_response.status

    def __repr__(self) -> str:
        return f"<Response [{self.status_code}]>"
