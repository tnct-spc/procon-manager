type ApiResponseLike = Response | { status?: number; data?: { message?: string } }

export interface ApiError {
  response?: ApiResponseLike
  error?: unknown
  message?: string
}

const extractMessage = (value: unknown): string | undefined => {
  if (!value) return undefined
  if (typeof value === 'string') return value
  if (typeof value === 'object' && 'message' in value && typeof value.message === 'string') {
    return value.message
  }
  return undefined
}

const hasResponseData = (
  response: ApiResponseLike,
): response is { status?: number; data?: { message?: string } } =>
  typeof (response as { data?: unknown }).data !== 'undefined'

export const getErrorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message
  }

  const apiError = error as ApiError
  const responseMessage =
    apiError.response && hasResponseData(apiError.response)
      ? extractMessage(apiError.response.data)
      : undefined
  const message =
    extractMessage(apiError.error) ?? responseMessage ?? extractMessage(apiError.message)

  return message || '操作に失敗しました'
}
