import axios from "axios";

const api = axios.create({
  baseURL: "https://procon-manager-item-manager-zcuq.shuttle.app/api/v1",
  headers: { "Content-Type": "application/json" },
});

api.interceptors.request.use((config) => {
  const token = localStorage.getItem("accessToken");
  if (token && config.headers) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export default api;
