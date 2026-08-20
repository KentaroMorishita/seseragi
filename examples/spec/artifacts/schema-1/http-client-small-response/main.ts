import { boundedShow as _ssrg_show_boundedShow, httpBuildErrorShow as _ssrg_show_httpBuildErrorShow, byteErrorShow as _ssrg_show_byteErrorShow, httpErrorShow as _ssrg_show_httpErrorShow, utf8DecodeErrorShow as _ssrg_show_utf8DecodeErrorShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, fromEither as _ssrg_effect_fromEither } from "@seseragi/runtime/effect"
import { parseUrl as _ssrg_http_client_parseUrl, withRequestHeader as _ssrg_http_client_withRequestHeader, request as _ssrg_http_client_request, post as _ssrg_http_client_post, sendBytes as _ssrg_http_client_sendBytes, defaultBodyLimit as _ssrg_http_client_defaultBodyLimit, responseBody as _ssrg_http_client_responseBody, type HttpBuildError as HttpBuildError, type HttpError as HttpError, type HttpUrl as HttpUrl, type Request as Request, type Method as Method, type Response as Response, type HttpBodyLimit as HttpBodyLimit } from "@seseragi/runtime/http-client"
import { fromInts as _ssrg_bytes_fromInts, type ByteError as ByteError, type Bytes as Bytes } from "@seseragi/runtime/bytes"
import { decodeUtf8 as _ssrg_text_decodeUtf8, type Utf8DecodeError as Utf8DecodeError } from "@seseragi/runtime/text"

type AppError =
  | { readonly tag: "BuildFailure"; readonly value: HttpBuildError }
  | { readonly tag: "BytesFailure"; readonly value: ByteError }
  | { readonly tag: "HttpFailure"; readonly value: HttpError }
  | { readonly tag: "TextFailure"; readonly value: Utf8DecodeError };
const BuildFailure = (value: HttpBuildError): AppError => ({ tag: "BuildFailure", value } as const);
const BytesFailure = (value: ByteError): AppError => ({ tag: "BytesFailure", value } as const);
const HttpFailure = (value: HttpError): AppError => ({ tag: "HttpFailure", value } as const);
const TextFailure = (value: Utf8DecodeError): AppError => ({ tag: "TextFailure", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<AppError> = _ssrg_show_boundedShow((value: AppError): string => { switch (value.tag) { case "BuildFailure": return "BuildFailure" + " " + _ssrg_show_httpBuildErrorShow.show(value.value); case "BytesFailure": return "BytesFailure" + " " + _ssrg_show_byteErrorShow.show(value.value); case "HttpFailure": return "HttpFailure" + " " + _ssrg_show_httpErrorShow.show(value.value); case "TextFailure": return "TextFailure" + " " + _ssrg_show_utf8DecodeErrorShow.show(value.value); } });
export const fetch = (urlText: string) => _ssrg_effect_flatMap(_ssrg_effect_mapError(BuildFailure, _ssrg_effect_fromEither(_ssrg_http_client_parseUrl(urlText))), (url: HttpUrl) => _ssrg_effect_flatMap(_ssrg_effect_mapError(BuildFailure, _ssrg_effect_fromEither(_ssrg_http_client_withRequestHeader("content-type", "application/octet-stream", _ssrg_http_client_request(_ssrg_http_client_post, url)))), (request: Request) => _ssrg_effect_flatMap(_ssrg_effect_mapError(BytesFailure, _ssrg_effect_fromEither(_ssrg_bytes_fromInts([115, 101, 115, 101, 114, 97, 103, 105]))), (body: Bytes) => _ssrg_effect_flatMap(_ssrg_effect_mapError(HttpFailure, _ssrg_http_client_sendBytes(_ssrg_http_client_defaultBodyLimit(undefined), body, request)), (response: Response) => _ssrg_effect_mapError(TextFailure, _ssrg_effect_fromEither(_ssrg_text_decodeUtf8(_ssrg_http_client_responseBody(response))))))))
