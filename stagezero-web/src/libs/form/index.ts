export type TFieldErrorItem = { message?: string }

export function toFieldErrors(errors: readonly unknown[]): TFieldErrorItem[] {
  return errors.map((error) => {
    if (typeof error === 'string') {
      return { message: error }
    }
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return { message: String(error.message) }
    }
    return { message: undefined }
  })
}
