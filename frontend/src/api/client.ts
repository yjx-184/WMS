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
/*  响应拦截器                                                         */
/*                                                                     */
/*  所有业务 API 响应格式为 `{code, data, message}`。                  */
/*  - code === 0: 成功，返回 response（调用方从 data.data 取数据）     */
/*  - code !== 0: 业务错误，弹出 message.error() 提示                  */
/*  - 网络/HTTP 错误: 弹出错误提示                                      */
/* ------------------------------------------------------------------ */

apiClient.interceptors.response.use(
  /* 成功响应 — 检查业务错误码 */
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
