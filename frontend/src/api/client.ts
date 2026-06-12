import axios from 'axios';
import { message } from 'antd';

const apiClient = axios.create({
  baseURL: '/api/v1',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
    Accept: 'application/json',
  },
});

/* ------------------------------------------------------------------ */
/*  Response interceptor                                               */
/* ------------------------------------------------------------------ */

apiClient.interceptors.response.use(
  /* Success handler — check business-level error code. */
  (response) => {
    const body = response.data;

    // Unified backend envelope: { code, data, message }
    if (body && typeof body.code === 'number' && body.code !== 0) {
      message.error(body.message || '请求失败');
      return Promise.reject(body);
    }

    return response;
  },

  /* Error handler — network / HTTP errors. */
  (error) => {
    if (error.response) {
      // Server responded with a non-2xx status
      const body = error.response.data;
      message.error(body?.message || `请求失败 (${error.response.status})`);
    } else if (error.request) {
      // Request was made but no response received
      message.error('网络错误，请检查连接');
    } else {
      // Something went wrong while setting up the request
      message.error(error.message || '请求异常');
    }

    return Promise.reject(error);
  },
);

export default apiClient;
