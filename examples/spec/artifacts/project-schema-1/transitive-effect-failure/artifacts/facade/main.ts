import { reject, type InputError } from "./provider.js"

export const rejectViaFacade = (input: string) => reject(input)
