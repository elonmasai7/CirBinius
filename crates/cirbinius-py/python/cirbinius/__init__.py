import ctypes
import json
import os
from typing import Any, Optional


def _find_lib() -> str:
    _lib_dir = os.path.dirname(os.path.abspath(__file__))
    _repo_root = os.path.abspath(os.path.join(_lib_dir, "..", "..", "..", ".."))
    dirs = [
        _lib_dir,
        os.path.join(_repo_root, "target", "release"),
        os.path.join(_repo_root, "target", "debug"),
        "/usr/local/lib",
        "/usr/lib",
    ]
    for d in dirs:
        p = os.path.join(d, "libcirbinius_py.so")
        if os.path.exists(p):
            return p
    raise RuntimeError("libcirbinius_py.so not found")


_lib = ctypes.CDLL(_find_lib())

_lib.cirbinius_client_new.argtypes = [
    ctypes.c_char_p,
    ctypes.c_uint16,
    ctypes.c_char_p,
]
_lib.cirbinius_client_new.restype = ctypes.c_void_p

_lib.cirbinius_client_free.argtypes = [ctypes.c_void_p]
_lib.cirbinius_client_free.restype = None

_lib.cirbinius_request.argtypes = [
    ctypes.c_void_p,
    ctypes.c_char_p,
    ctypes.c_char_p,
    ctypes.c_char_p,
]
_lib.cirbinius_request.restype = ctypes.c_char_p

_lib.cirbinius_free_string.argtypes = [ctypes.c_char_p]
_lib.cirbinius_free_string.restype = None

_lib.cirbinius_get_last_error.argtypes = []
_lib.cirbinius_get_last_error.restype = ctypes.c_char_p


class CirbiniusError(Exception):
    pass


class CirbiniusClient:
    def __init__(
        self,
        host: str = "127.0.0.1",
        port: int = 8080,
        api_key: Optional[str] = None,
    ):
        host_b = host.encode("utf-8")
        key_b = api_key.encode("utf-8") if api_key else None
        self._handle = _lib.cirbinius_client_new(host_b, port, key_b)
        if not self._handle:
            self._raise_error()

    def close(self) -> None:
        if self._handle:
            _lib.cirbinius_client_free(self._handle)
            self._handle = None

    def __del__(self) -> None:
        self.close()

    def _request(self, method: str, path: str, body: Any = None) -> Any:
        method_b = method.encode("utf-8")
        path_b = path.encode("utf-8")
        body_b = json.dumps(body).encode("utf-8") if body is not None else None

        result = _lib.cirbinius_request(self._handle, method_b, path_b, body_b)
        if not result:
            self._raise_error()

        text = result.decode("utf-8")
        _lib.cirbinius_free_string(result)
        return json.loads(text)

    def _raise_error(self) -> None:
        err = _lib.cirbinius_get_last_error()
        msg = err.decode("utf-8") if err else "unknown error"
        if err:
            _lib.cirbinius_free_string(err)
        raise CirbiniusError(msg)

    # ---- Health ----

    def health(self) -> dict:
        return self._request("GET", "/health")

    # ---- Projects ----

    def list_projects(self) -> list[dict]:
        return self._request("GET", "/api/v1/projects")

    def create_project(self, name: str, description: str = "") -> dict:
        return self._request("POST", "/api/v1/projects", {"name": name, "description": description})

    def get_project(self, project_id: str) -> dict:
        return self._request("GET", f"/api/v1/projects/{project_id}")

    def update_project(self, project_id: str, **kwargs) -> dict:
        return self._request("PATCH", f"/api/v1/projects/{project_id}", kwargs)

    def delete_project(self, project_id: str) -> dict:
        return self._request("DELETE", f"/api/v1/projects/{project_id}")

    # ---- Uploads ----

    def list_uploads(self, project_id: str) -> list[dict]:
        return self._request("GET", f"/api/v1/projects/{project_id}/uploads")

    def upload_file(self, project_id: str, filename: str, content_type: str, data: bytes) -> dict:
        query = f"filename={filename}&content_type={content_type}"
        path = f"/api/v1/projects/{project_id}/uploads?{query}"
        return self._request("POST", path, data)

    def get_upload(self, project_id: str, upload_id: str) -> dict:
        return self._request("GET", f"/api/v1/projects/{project_id}/uploads/{upload_id}")

    def delete_upload(self, project_id: str, upload_id: str) -> dict:
        return self._request("DELETE", f"/api/v1/projects/{project_id}/uploads/{upload_id}")

    # ---- Jobs ----

    def list_jobs(self, project_id: str) -> list[dict]:
        return self._request("GET", f"/api/v1/projects/{project_id}/jobs")

    def _create_job(self, project_id: str, job_type: str, params: dict = None) -> dict:
        return self._request("POST", f"/api/v1/projects/{project_id}/{job_type}", params or {})

    def create_compile_job(self, project_id: str, params: dict = None) -> dict:
        return self._create_job(project_id, "compile", params)

    def create_prove_job(self, project_id: str, params: dict = None) -> dict:
        return self._create_job(project_id, "prove", params)

    def create_verify_job(self, project_id: str, params: dict = None) -> dict:
        return self._create_job(project_id, "verify", params)

    def create_analyze_job(self, project_id: str, params: dict = None) -> dict:
        return self._create_job(project_id, "analyze", params)

    def create_conformance_job(self, project_id: str, params: dict = None) -> dict:
        return self._create_job(project_id, "conformance", params)

    def get_job(self, project_id: str, job_id: str) -> dict:
        return self._request("GET", f"/api/v1/projects/{project_id}/jobs/{job_id}")

    def cancel_job(self, project_id: str, job_id: str) -> dict:
        return self._request("POST", f"/api/v1/projects/{project_id}/jobs/{job_id}/cancel")

    def get_job_logs(self, project_id: str, job_id: str) -> list[dict]:
        return self._request("GET", f"/api/v1/projects/{project_id}/jobs/{job_id}/logs")

    # ---- Artifacts ----

    def list_artifacts(self, job_id: str) -> list[dict]:
        return self._request("GET", f"/api/v1/jobs/{job_id}/artifacts")

    def get_artifact(self, job_id: str, artifact_id: str) -> dict:
        return self._request("GET", f"/api/v1/jobs/{job_id}/artifacts/{artifact_id}")

    def download_artifact(self, job_id: str, artifact_id: str) -> bytes:
        result = self._request("GET", f"/api/v1/jobs/{job_id}/artifacts/{artifact_id}/download")
        if isinstance(result, str):
            return result.encode("utf-8")
        return bytes(result)

    # ---- Admin ----

    def get_stats(self) -> dict:
        return self._request("GET", "/api/v1/admin/stats")

    def list_api_keys(self) -> list[dict]:
        return self._request("GET", "/api/v1/admin/api-keys")

    def create_api_key(
        self,
        name: str,
        project_id: Optional[str] = None,
        permissions: Optional[list[str]] = None,
        expires_in_days: Optional[int] = None,
    ) -> dict:
        body: dict[str, Any] = {"name": name}
        if project_id:
            body["project_id"] = project_id
        if permissions:
            body["permissions"] = permissions
        if expires_in_days:
            body["expires_in_days"] = expires_in_days
        return self._request("POST", "/api/v1/admin/api-keys", body)

    def delete_api_key(self, key_id: str) -> dict:
        return self._request("DELETE", f"/api/v1/admin/api-keys/{key_id}")

    def check_auth(self) -> dict:
        return self._request("POST", "/api/v1/admin/auth")
