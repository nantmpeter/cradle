export interface Setup2FaResponse {
  qr_code_base64: string
  secret: string
  recovery_codes: string[]
}

export interface Enable2FaRequest {
  code: string
}

export interface Disable2FaRequest {
  password?: string
  code?: string
}

export interface Verify2FaRequest {
  temp_token: string
  code: string
}
