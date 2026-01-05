"""Health endpoint tests for Auth Service."""

import pytest
from fastapi.testclient import TestClient

from app.main import app


@pytest.fixture
def client():
    """Test client fixture."""
    return TestClient(app)


def test_health_endpoint(client):
    """Test health check endpoint returns 200 OK."""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert data["service"] == "auth-service"
    assert "version" in data


def test_health_endpoint_structure(client):
    """Test health check response has correct structure."""
    response = client.get("/health")
    data = response.json()
    assert set(data.keys()) == {"status", "service", "version"}
    assert isinstance(data["status"], str)
    assert isinstance(data["service"], str)
    assert isinstance(data["version"], str)


def test_root_endpoint(client):
    """Test root endpoint returns service information."""
    response = client.get("/")
    assert response.status_code == 200
    data = response.json()
    assert "service" in data
    assert "version" in data
    assert "endpoints" in data


def test_security_headers(client):
    """Test security headers are present in responses."""
    response = client.get("/health")
    assert "x-content-type-options" in response.headers
    assert response.headers["x-content-type-options"] == "nosniff"
    assert "x-frame-options" in response.headers
    assert response.headers["x-frame-options"] == "DENY"
