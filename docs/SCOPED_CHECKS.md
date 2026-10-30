# Scoped checks

Seseragiの検証は、変更範囲に対応するscoped laneを先に実行します。`bun run check`
は、各laneを日常作業で無条件に繰り返すためのコマンドではなく、repository-wideな
最終統合gateです。

## Lane matrix

native DecimalのtoIntExact/fromFloat/toFloatとconversion error constructorはbackendの
`--test decimal_conversion`、runtimeの`decimal_conversion_tests`、CLIの
`basic_decimal_conversion_`で確認します。Float bitからexactな係数/scaleを作ってから
6種の丸めを適用し、巨大指数の整数化はrange確認後に限定して展開します。exactなFloat最大値の
直上拒否、signed underflow、256有限Floatの17桁往復と独立Python Decimal期待値も確認します。
保存/partial/generic・opaque2module/error alias、元raw TS両profile strict・fresh Bun/Node/native、
cold/Failure/Defect/finalizer/root0と元入力を保持し、新program WASMは受け入れません。
最終SDKと共有runtimeの全体gateはRelease batchへ集約します。

native DecimalContextのcontext/precision/roundingとprecision error constructorは
backendの`--test decimal_context`、runtimeの`decimal_context_tests`、CLIの
`basic_decimal_context_`で確認します。既存のstd/number RoundingMode 6種を共有し、
safe Int precision・nonpositive error・保存/partial/generic・opaque2moduleと標準aliasを保持します。
最後のrounding引数を評価してから結果を作り、cold/Failure/Defect/finalizer/Ref cycle/root0を
元raw TS両profile strict・fresh Bun/Node/nativeで比較します。元Decimalのparse/fromInt/Eq/Ord/Hash/
表示も確認し、新program WASMは受け入れません。最終SDKと全体gateはRelease batchへ集約します。

native BigInt算術/Zero/Oneとchecked APIはbackendの`--test big_arithmetic`、runtimeの
`big_int_arithmetic_tests`、CLIの`basic_big_arithmetic_`で確認します。binary32のexactな
加減乗除・剰余・Int指数、0方向の商と被除数と同符号の剰余、typed zero/negative error、
保存/generic/opaque2module、元raw TS両profile strict・fresh Bun/Node/native、cold/Failure/
Defect/finalizer/Ref cycle/root0を保持します。signed i128と独立Python多word期待値も比較し、
新program WASMは受け入れません。共有operator validatorの変更では元scalar/partial checksを使います。
canonical ownerのalias照合と算術sectionの最初の引数型もdriverで確認し、previewしたanalysisは
再利用します。最終sourceの正規SDKと共有frontend/runtimeのrepository-wide gateはRelease batchへ集約します。

native BigIntのEq/Ord/Hash/Show/Debugはbackendの
`--test big_int_keys --test big_int_ord --test big_int_display`、
runtimeの`big_int_dictionary_tests`と`big_int_tests`、CLIの`basic_big_keys_`で確認します。
binary32 limbと必要時のdecimal cacheでEq/Ord/HashをO(n)にし、parse/formatはspecのO(d²)内で
元10万桁入力と正確なradix/Int変換を保持します。TSと同じsigned FNV hash・実衝突、OrdのEq継承、
保存/generic・opaque2module・元raw TS両profile strict・fresh Bun/Node/native、cold/Failure/Defect/
finalizer/Ref cycle/root0、位置付き偽造拒否と未使用program WASM拒否を確認します。
算術/Zero/Oneは別leafで続け、最終SDKと全体gateはRelease batchへ集約します。

native BigIntのradix変換/toIntはbackendの`--test big_radix --test basic_big_int`、runtimeの
`big_int_tests`、CLIの`basic_big_radix_`で確認します。binary32 limbで2〜36進を正確に変換し、
safe Int外をtyped errorにします。元10万桁decimal入力、canonical spellingとradix版の
leading zero、UTF-8位置、保存/部分適用・opaque2module・元raw TS両profile strict・fresh
Bun/Node/native・cold/Failure/Defect/finalizer/root0を保持します。元位置の偽造拒否と未使用
program WASM拒否も確認し、最終SDKと共有runtimeの全体gateはRelease batchへ集約します。

native Mathの定数・残りの単項関数・atan2/hypotはbackendの`--test math_remaining`、runtimeの
`math_tests`、CLIの`basic_math_remaining_`で確認します。定数はvalue、atan2はyが先で、
hypotは中間二乗のoverflow/underflowを避けます。NaN/Infinity/±0とspecの有限代表点の許容誤差、
保存/部分適用・opaque2module・元raw TS両profile strict・fresh Bun/Node/native、cold/Failure/
Defect/finalizer/root0と位置付き偽造拒否を確認します。元の4 Math操作も実行比較し、
新しいprogram WASM受入は追加しません。最終SDKと全体gateはRelease batchへ集約します。

native Intのchecked/saturating演算とhelperはbackendの`--test int_checked --test int_text
--test int_parse`、runtimeの`int_checked_tests`、CLIの`basic_int_checked_`で確認します。
safe Int範囲の正確な演算、右引数先行の順序、巨大指数でも有界なpower、typedな除算/
power errorとclampのDefectを保ちます。保存/部分適用・opaque 2module・元raw TS両profile
strict・fresh Bun/Node/native・cold/Failure/finalizer/root0、位置付き偽造拒否と未使用program
WASM拒否を確認します。最終SDKと共有runtimeの全体gateはRelease batchへ集約します。

native Intの基数変換・format・上下限はbackendの`--test int_text --test int_parse`、
runtimeの`int_parse_tests`、CLIの`--test basic_standard basic_int_radix_`で確認します。
radix 2〜36、safe Int範囲、符号・leading zero・大小文字、UTF-8 byte位置と最初の失敗を
TSに揃え、既存十進parseのcanonical spellingを保ちます。保存/部分適用・generic関数・
opaque 2module package、元raw TS両profile strict/skipLibCheck:false、fresh Bun/Node/native、
cold/Failure/Defect/finalizer/root0、偽造owner/kind/型/辞書/段階の位置付き拒否を確認します。
新program WASMは未使用の保存関数でも拒否し、最終SDK/gateはRelease batchへ集約します。
標準curried関数を返す元Int入力のTS7006回帰は両profileの元raw TS strictでも確認し、
driverの`--test strict_partials --test generic_call_instantiation --test hkt_erasure`で
checkedな型とsource段階を保持します。共有lowering修復の正規WASMも最終batchで更新します。

native Decimal Ordと比較演算はbackendの`--test decimal_ord --test decimal_keys
--test basic_decimal --test method_references`、runtimeの`decimal_tests`、CLIの
`basic_decimal_ord_`で確認します。符号/zero/任意精度の桁指数/仮想末尾zeroを比較し、
1000桁のscaleを展開せず、OrdのEq継承・保存/部分適用・opaque2module packageを保ちます。
元zero回帰入力はbyte不変で、raw TS両profile strictとfresh Bun/Node/native、cold/Failure/
引数・戻りcallback Defect/finalizer/Ref cycle/root0、method22とbinary32通りの偽造拒否、
未使用compare/4比較演算のprogram WASM拒否を確認します。最終SDK/gateはRelease batchです。

TS Decimal zero比較の修復はruntimeの`decimal.test.ts`と`decimal-zero-order.test.ts`、CLIの
`--test decimal_zero_order`で確認します。元SSRG入力を保ち、development/release両profileの
fresh Bun/Nodeでzero/負zero・巨大正負scale・正負/同値/非zero順序を比較します。
native Ordの追加は別leafとし、runtimeの最終integrationはRelease batchへ集約します。

native Decimal Eq/Hashはbackendの`--test decimal_keys --test basic_decimal`、runtimeの
`decimal_tests`、CLIの`basic_decimal_keys_`で確認します。正規化した任意精度coefficient/scale、
TSと同じsigned little-endian 32-bit limb hash、等値spellingと実hash衝突を持つMap/Set、
同名nominalとimported opaque key、保存generic methodとsource段階を保ちます。元raw TS両profile
strictとfresh Bun/Node/native、cold/typed Failure/Defect/finalizer/Ref cycle/root0、偽造owner/
kind/引数/結果/辞書/dispatch42通りと未使用program WASM拒否を確認します。Ord/算術は別leafです。

native Set.mapはbackendの`--test basic_set_map`、runtimeの`set_tests`、CLIの
`--test basic_standard basic_set_map_`で確認します。Map.mapKeysWithと実装を共有し、
出力BのEq/Hash、mapper→Hash→必要なEq、最初の出力代表/順・seed・snapshotを保持します。
入力keyと元containerの非保持、通常2module opaque packageと保存imported generic partial、
元raw TS両profile strictとfresh Bun/Node/native、cold/typed Failure/mapperと戻りcallback内
Defect/finalizer/Ref cycle/root0、Hash/Eq Defectと循環辞書のunwind、偽造contract22通りと
未使用program WASM拒否を確認します。Map runtime/対象backendも回帰確認し、最終SDK/gateはbatchです。

制約付きglobal partial/戻り関数とTSのcallback型はbackendの`--test returned_evidence_stages
--test strict_partials`、driverの`--test strict_partials --test generic_call_instantiation
--test hkt_erasure`、CLIの元`basic_map_upsert_`入力で確認します。元入力の両profile raw TS strictと
Bun/Node/nativeを比較し、確定済みcallback型・宣言のsource境界・辞書位置を保ちます。
残り引数/結果/owner/kind/辞書の偽造を拒否し、共有loweringのSDK/全体gateは最終batchで確認します。

native Map.upsert/mergeWith/mapKeysWithはbackendの`--test basic_map_upsert`、runtimeの`map_tests`、
CLIの`--test basic_standard basic_map_upsert_`で確認します。Hash一度→Eq検索→update一度の順、
mergeのleft代表/順とright-only末尾、key変換と衝突resolveのsource順・最初の出力代表を保ちます。
mapKeysWithの辞書はinput K1ではなくoutput K2を検証し、seedとV型を保持します。
元raw TS両profile strictとfresh Bun/Node/native、通常2 module packageのopaque input/output keyと
保存generic partial、cold/Failure/update・merge・key・collision callback内Defect/finalizer/Ref cycle/
root0、owner/kind/型/辞書/段階の元位置付き拒否と未使用program WASM拒否を確認します。
insertの置換/lookupを共有するため既存Map/Set runtimeと対象CLIも回帰確認します。SDK/全体gateは最終batchです。

native Map.filter/mapValuesはbackendの`--test basic_map_operations --test basic_map`、runtimeの
`map_tests`、CLIの`--test basic_standard basic_map_operations_`で確認します。filterのcached hash/
seedとbulk再構築はSetと共有するため`set_tests`とCLIの`basic_set_operations_`も回帰確認します。
挿入順のcallback、元key代表・snapshot・空入力、mapValuesのvalue型変更とbucket共有、
除外key/value・置換前value・元containerの非保持を保ちます。通常2 module packageのopaque値・
imported generic partialと保存callback、元raw TS両profile strictとfresh Bun/Node/native、
cold/Failure/predicate・mapper内Defect/finalizer/Ref cycle/root0を比較します。
callback/結果/owner/kind/型/辞書/段階と未使用保存関数のprogram WASMを拒否します。
公開面はstandard.rsとspecに従い、reduceはstd/map関数として追加しません。SDK/全体gateは最終batchです。

native Setのfilter/union/intersection/difference/isSubsetOf/toListはbackendの
`--test basic_set_operations --test basic_collections`、runtimeの`set_tests`、CLIの
`--test basic_standard basic_set_operations_`で確認します。選択済みEq/Hash、挿入順と元key代表、
snapshot、predicate順とsubsetのshort-circuit、filter系のcached hashとseedを保持します。
collision bucketを一度ずつ構築し、除外要素と元containerを隠れて保持しません。
通常2 module packageのopaque key/imported辞書と保存generic partial、元raw TS両profile strictと
fresh Bun/Node/native、cold/Failure/predicate内Defect/finalizer/Ref cycle/root0を比較します。
Hash/Eq内Defectと循環辞書のunwindはruntimeで各binary演算を確認し、owner/kind/要素/結果/
辞書/段階と未使用保存関数のprogram WASMを拒否します。SDK/全体gateは最終batchです。

native text.startsWith/endsWithはbackendの`--test basic_standard`、runtimeの`text_tests`、
CLIの`--test basic_standard basic_text_boundaries_`で確認します。prefix/suffix先・本文後の
signature、保存/部分適用・generic caller、空文字・Unicode・結合文字・NULと非正規化を保ちます。
通常2 module packageのopaque値、元raw TS両profile strict/skipLibCheck:falseとfresh
Bun/Node/native、cold/Failure/引数内Defect/finalizer/Ref cycle/root0を比較します。
不正な引数/結果/kind/段階/辞書と未使用保存関数のprogram WASMを拒否し、SDK/全体gateは
共有runtimeへの変更をまとめた最終batchで確認します。

generic callに渡す先頭空の入れ子Array/Listはdriverの`--test nested_empty_call_inference`、
既存の`generic_call_instantiation`/`hkt_erasure`/`strict_partials`、semanticsのlib、CLIの
`inferred_nested_empty_collection_`で確認します。元入力をbyte不変で保持し、raw TS両profile
strict/skipLibCheck:falseとfresh Bun/Node/nativeで比較します。既存rank-1 solverへcollection
literalも渡し、後続siblingの型を通常checkerより先に共有します。nominal/collectionの不一致と
UTF-8範囲、全要素が空でcontextもない場合の拒否、明示型引数対照を保ちます。
shared frontendの変更なので正規compiler WASMと全体gateは最終batchで更新・確認します。

native Array/Listのconcatはbackendの`--test basic_sequence`、runtimeの`sequence_tests`、
CLIの`--test basic_standard basic_concat_`で確認します。内外両方のcanonical collectionと
要素型・kind・保存generic関数を照合し、順序・空入力・一段だけの連結を保ちます。
通常2 module packageのopaque要素とcapture、元raw TS両profile strictとfresh Bun/Node/native、
cold replay・返されたcallback内Defect・Failure・finalizer/Ref cycle/root0を確認します。
結果は元の外側/内側containerを隠れて保持せず、新program WASMは未使用の保存関数でも
拒否します。正規SDKと全体gateは最終batchへ集約します。

tagged ADTの派生JSON codecはbackendの`--test basic_json_derived_adts`、runtimeの
`json_derived_adt_tests`、CLIの`--test basic_standard basic_json_derived_adts_`で確認します。
payloadなしとUnit payload、generic/recursive、opaque/importedと保存methodを区別します。
tag→unknown field→payloadの検査順・path、選択branchだけの遅延辞書構築、canonicalな
variant owner/kind/子evidence、元UTF-8位置の拒否、新program WASM拒否を保ちます。
元31ケースと既存canonical spec、通常2 module packageをraw TS両profile strictとfresh
Bun/Node/nativeで比較し、cold/Failure/Defect/finalizer/Ref cycle/root0、encode/decode両方の
128 frame host limitを確認します。正規WASMとrepository-wide gateは最終batchで行います。

transparent newtypeの派生JSON codecはbackendの`--test basic_json_derived_newtypes`とCLIの
`--test basic_standard basic_json_derived_newtypes_`で確認します。generic/recursive・保存method・
opaque/importedの通常2 module packageと、tagを加えないwireと失敗path、cold/Failure/Defect/
finalizer/Ref cycle/root0、encode/decode両方の128 frame host limitを比較します。
releaseのchecked newtype constructorは既存のprivate representation変換を使います。
driverの`--test newtype_literal_profiles`、CLIの`newtype_literal_representations_`で元の
record/array/tuple literalをraw TS両profile strict/skipLibCheck:false、fresh Bun/Node/nativeで
通し、別nominalや誤ったrepresentationのsource拒否を保ちます。正規WASMと全体gateは最終batchです。

named structの派生JSON codecはbackendの`--test basic_json_derived_structs`、CLIの
`--test basic_standard basic_json_derived_structs_`で確認します。declaration順・generic/recursive
struct・保存method・opaque/importedの通常2 module package、unknown先行拒否とnested path、
cold/Failure/Defect/finalizer/Ref cycle/root0、128 frameのhost limitを元raw TS strictと
fresh Bun/Node/nativeで比較します。短いderived identityはcanonicalなnominal headから
検証し、一般のuser辞書や同名traitへの信頼を広げません。新program WASMは拒否します。
struct内のrecord型と後続fieldはsyntaxの`nested_record_types_keep_their_struct_fields`、
driverの`--test struct_construction_kind`、CLIの`nested_record_struct_fields_`で確認します。
#1012の元sourceとUTF-8範囲・型errorを保ち、shared parser修復の正規WASMと全体gateは
最終batchで行います。

JSON recordのprototype名のfieldはPlaygroundの`json-record-runtime.test.ts`とCLIの
`--test basic_standard json_record_prototype_field_`で確認します。structural/derived decoderが
own data propertyを作り、通常objectのprototype、optional不在、unknown先行拒否を保持します。
#1010の元sourceはbyte不変で保存し、両profileのraw TS strict・fresh Bun/Node/nativeで比較します。

native JSONのArray/List/Maybe/Either/Tuple/Map/Set/closed record codecはbackendの
`--test basic_json_containers`、runtimeの`json_container_codec_tests`、CLIの
`--test basic_standard basic_json_containers_`で確認します。optional fieldの不在とnull、
tuple長の先行検査、recordのunknown先行拒否、全要素decode後のEq/Hash、重複keyの
最初の代表と最後の値を保持します。保存generic method・入れ子・opaque要素とimported
user辞書を通常2 module package、元raw TS両profile strict/skipLibCheck:false、fresh
Bun/Node/nativeで比較します。cold再実行・Failure/encoder内Defectのfinalizer/Ref cycle/root0、
128 codec frameのhost limitと復元、owner/kind/子辞書と元UTF-8位置の拒否も確認します。
新program WASMは未使用の辞書でも拒否します。共有native runtimeへの追加なので、
最終batch候補で正規compiler WASMの更新とrepository-wide統合gateを行います。

importしたcallable alias内のPrelude sumから推論した型引数は、driverの
`--test generic_call_instantiation --test hkt_erasure --test strict_partials`と、
CLIの`--test basic_standard inferred_json_decoder_package`で確認します。#1003の元入力と
opaque module境界を保持し、元生成TS両profile strict/skipLibCheck:false、fresh Bun/Node/nativeを
通します。Preludeのcanonical owner・arityを登録から選び、明示source binding・同名nominal・
量化したconstructor slotを優先します。登録のない同名typeをPreludeと推測しません。
shared frontendの変更なのでsemantics/loweringの対象確認後、canonical compiler WASMを更新し、
最終候補でrepository-wide gateを通します。

native JSON decoderのfield/optionalField/index/array/record/oneOf/map/flatMapはbackendの
`--test basic_json_decoders`、runtimeの`json_decoder_tests`、CLIの
`--test basic_standard basic_json_decoders_`で確認します。保存/部分適用と戻りdecoderの
連続適用、canonical owner/kind/引数/結果と無辞書のsource段階を検証します。missingとchild
failureのpath、recordのunknown先行拒否/指定順/重複指定、oneOfの最後の失敗と空入力、
short-circuitとflatMapの元Jsonを保ちます。元raw TS両profile strict、fresh Bun/Node/native、
通常2 module package、cold/Failure/mapper内Defect/finalizer/Ref cycle/root0を比較します。
nativeの動的合成は同時128 decoder frameをhost limitとし、stack abortにせずDefectにします。
このnative固有境界は元入力のstrict比較とは別に、finalizer/root0とframe復元を確認します。
新program WASMは拒否します。compiler WASMの依存を変えない追加でも、最終候補で正規Linux
freshnessと全体統合gateを通す必要があります。

native JSONのBool/String/Int/Decimal/Unit/Jsonの12標準codec辞書とstring wrapperは
backendの`--test basic_json_codecs`、runtimeの`json_codec_tests`、CLIの
`--test basic_standard basic_json_codecs_`で確認します。exact Decimalから整数性とsafe Int
範囲を判定し、巨大指数をFloatや巨大Stringへ展開しません。wrong-type message、root pathと
syntax/domain error、公開error constructor、generic/保存methodと明示user辞書、通常2 module
packageを元raw TS両profile strict/skipLibCheck:false、fresh Bun/Node/nativeで比較します。
cold/Failure/encoder内Defectとfinalizer/Ref cycle/root0、不正owner/kind/child/段階の位置付き
拒否を保ちます。新しいprogram WASM受入は行いません。
DecodeErrorの公開struct登録とgeneric JSON辞書のTS型・確認済み型引数の保持はshared frontendの
修正です。driverの`--test json_decode_error --test struct_construction_kind`を通し、元fixtureと
assertionを維持します。統合時は正規compiler WASMを更新し、全体gateを実行します。

native Array/Listのsequence基本22操作はbackendの`--test basic_sequence`、runtimeの
`sequence_tests`、CLIの`--test basic_standard basic_sequence_`で確認します。
findIndex/takeWhile/dropWhileのcallback順とshort-circuit、zipの左右型と短い入力、
reduceRightの右からの評価、空入力・singleton・非正/巨大sizeとSizeError constructorを
検査します。windowやprefixの結果に除外した要素を保持せず、Array lastのO(1)と
Listの含まれるsuffix共有を保ちます。元raw TS両profile strict、fresh Bun/Node/native、
通常2 module path package、保存関数・generic partial・cold replay・Failure/Defect後の
finalizer/Ref cycle/root0と、canonical owner/kind/要素/段階/辞書の位置付き拒否を確認します。
新しいprogram WASMとSizeError標準Eq/Show、sort/groupBy等は追加しません。
共有runtimeの統合には全体gateと正規compiler WASMの更新も必要です。

native JSON parse/stringifyとJson/JsonPathSegment/JsonParseErrorの公開constructorは、
backendの`--test basic_json`、runtimeの`json_tests`、CLIの
`--test basic_standard basic_json_`で確認します。Decimalの正確な係数・scaleを保ち、
objectは標準の挿入順Mapを使います。UTF-8 byte位置とsyntax message、decoded field名の
重複とrootからのpath、Unicode escape、通常2 module path packageと実ファイル変換を
元raw TS両profile strict・fresh Bun/Node/nativeで比較します。構文/重複/UTF-8の失敗時は
既存出力を保持します。cold再実行・Failure/Defectのfinalizerとroot0、Jsonを捕捉する
Ref cycle、host limitを確認します。parser/stringifierはheap frameで深さ20,000を検査します。
nativeの内部Map seedはobject構築時に解決します。TS process entryは起動時にseedを選び
環境overrideを消すため、不正環境seedのfinalizer検査はnative固有です。
JsonEncode/JsonDecode辞書やdecoder combinator、新program WASMは追加しません。
共有native runtimeの統合時は全体gateも通します。

native Decimal parse/fromInt/Showはbackendの`--test basic_decimal`、runtimeの
`decimal_tests`、CLIの`--test basic_standard basic_decimal_`で確認します。
係数とscaleを任意精度の符号付き十進列として保ち、指数をFloatや固定幅整数へ丸めません。
厳密なsyntax・最初の不正UTF-8位置・負zero/末尾zeroの正規化・canonical表記、
opaque値と保存formatterを渡す通常2 module path package、raw TS両profile strictと
fresh Bun/Node/nativeを比較します。巨大指数はparseでき、host limitを超える表記だけを
Defectにします。cold再実行・Failure/Defect後のfinalizer/root0、Ref cycleと
owner/kind/source段階の拒否も確認します。算術・丸め・全標準辞書や新program WASMは
追加しません。共有native runtimeの統合時は全体gateも通します。

native Set empty/singleton/contains/insert/remove/toArray/size/isEmptyと、Mapの
containsKey/entriesはbackendの`--test basic_collections`、runtimeの`set_tests`と
`map_tests`、CLIの`--test basic_standard basic_collections_`で確認します。
Setは専用tagでMapのpersistent storageを共有し、canonicalな型owner・kind・要素型と
選択済みEq/Hash、genericな部分適用のsource段階を検査します。順序・元key代表・snapshot、
Unicode/Unit、Mapのkey/value対応、通常2 module path package、raw TS両profile strict、
fresh Bun/Node/nativeを比較します。Set経由のRef cycle、列挙から取得するcallback、
cold再実行・Failure/Defect後のfinalizerとroot0、削除済みkeyの非保持を確認します。
Setを使うprogramも不正seedをアプリ開始前に拒否します。新10 APIのprogram WASM受入や
Map/Set全APIは追加しません。共有native runtimeの統合時は全体gateも通します。

native Map empty/singleton/get/insert/remove/keys/values/size/isEmptyはbackendの
`--test basic_map`、runtimeの`map_tests`、CLIの`--test basic_standard basic_map_`で確認します。
canonical owner/key/value・Eq/Hash・caller scopeと部分適用の残りsource段階を検査し、
元raw TS両profile strict、fresh Bun/Node/native、通常2 module path packageを比較します。
basic APIのpackageは`.gitattributes`でmanifest/sourceをLFに固定します。
Windowsの`core.autocrlf=true`相当のfilterでも元Git bytesとlock digestを保ち、
checkout差を隠すためのlock再生成やdigest除外は行いません。
挿入順・置換前keyの保持・collision callback・古いsnapshot・全signed 64-bit seed、
boundedなpath共有、削除済みkey/valueの非保持、cold再実行・Failure/Defect・Ref cycleの
回収を保ちます。OS entropyが取得できない場合は代替seedを作らず、固定seedの不正値も
アプリ開始前に拒否します。新9 APIのprogram WASM受入とMap全APIは追加しません。

ADT/newtype・primitive・opaque aliasをstruct構築構文でcastしてしまう既存不具合はdriverの
`--test struct_construction_kind`とsemanticsで確認します。元のMap入力を変更せず
拒否対照に保存し、両profileのcompile/analysisで同じSES-T0101とUTF-8位置を確認します。
通常struct・struct alias・括弧付きrecord payloadのconstructor適用は保持します。
aliasの固定・並替・重複した型引数、必須fieldと型、opaqueな表現の非公開境界も検査します。
共有frontendの修復のためcanonical WASMを再生成し、統合時に全体gateを通します。

recordのvarianceはdriverの`--test variance`でも確認します。ADTへのrecord payloadは
`Box ({ ... })`として実際にconstructorへ渡し、field順の同一性と型引数内のwidth拒否を保ちます。
旧`Box { ... }`の元入力はbyte不変のfixtureに残し、structを不正castできないことと
compile/analysisの同じ位置付き診断を両profileで確認します。通常native CIにもこの二つの
driver suiteを含め、統合時は全体gateの成功を確認します。

native Hashの標準Int/Bool/Char/String/Unit辞書はbackendの`--test basic_hash`、
runtimeの`hashing_tests`、CLIの`--test basic_standard basic_hash_`で確認します。
public hashにseedを混ぜず、safe Int・Unicode scalar FNV・選択済みの辞書・
generic/保存methodのA -> Int契約を保持します。偽のowner/kind/child/段階/結果を
元のUTF-8位置で拒否します。正式112で既に通るuser Hashのprogram WASMは元入力を
保持してBun/Nodeで実行し、新しいprimitive辞書は未使用の制約でも受け入れません。
共有runtimeの変更では既存basic APIと実program WASMも確認し、統合時に全体gateを通します。

native BigInt parse/format/fromIntはbackendの`--test basic_big_int`、runtimeの
`big_int_tests`、CLIの`--test basic_standard basic_big_int_`で確認します。
任意桁数をFloatや固定幅整数へ丸めず、符号・zeroの正規化、strict decimal構文と
最初の不正UTF-8 byte位置、canonical owner/kind/source stageを保ちます。
元raw TS両profileのstrictとfresh Bun/Node/native、通常path packageのopaque値・
保存formatter、Ref/cold再実行・Failure/Defect後のfinalizer/root0を比較します。
radix指定・BigInt算術と全辞書、Decimal、program WASMの新しい受入は含めません。

nativeのText split/trim/contains/replaceとMath sqrt/sin/cos/logは、backendとCLIの
`--test basic_standard`、runtimeの`math_tests`/`text_tests`で確認します。
Unicode scalar・White_Space・literal replacement、canonical owner/kind/source stageの拒否、
数学関数の特殊値と仕様の許容誤差、元のraw TS両profileのstrict、fresh Bun/Node/native、
saved functionのcold再実行とFailure/Defect後のfinalizer/root0、通常path packageの
CSV変換・不正入力時の旧出力保持を確認します。旧sqrt拒否sourceは成功対照へ保持し、
未対応cbrtとList comprehensionの位置付き拒否を残します。新8 APIのprogram WASM受入は
追加せず、target-specific proofを欠く入力を位置付きで拒否します。

native Bytes fromInts/toInts/get/toIntはbackendの`--test basic_bytes`、runtimeの
`bytes_tests`、CLIの`--test basic_standard basic_bytes_`で確認します。
全unsigned byte・空入力・最初の範囲外Int・負/大index・入力不変・独立storage、
canonical owner/kind/source stageと正確な位置、元raw TS両profileのstrict、
fresh Bun/Node/native、Refに保存したbufferと関数のcold再実行・Failure/Defect後の
finalizer/root0を保ちます。新しいprogram WASM ABIやByte/Bytesの全標準辞書、
残るBytes APIを受け入れたことにはしません。

native Array filter/find/get/length/take/dropはbackendの`--test basic_array`、runtimeの
`array_operation_tests`、CLIの`--test basic_standard basic_array_`で確認します。
genericな要素型・canonical owner/kind・保存/部分適用とsource stageを検証し、
元のraw TS両profileのstrict、fresh Bun/Node/nativeと通常path packageを比較します。
filterの順序、findの途中停止、空入力、負/大indexとcount、入力不変と独立storage、
関数要素のcapture・Ref cycle、cold再実行・Failure/Defect後のfinalizer/root0を保ちます。
receiver-firstな既存Array index ABIを変えず、新6 APIのprogram WASMは位置付きで拒否します。

native Int infix `>` / `>=`の修復はbackendの`--test comparison_partials`とCLIの
`--test basic_standard basic_array_infix_descending_`で確認します。
正式112で拒否された元のArray filterのsource、整数境界と真偽、canonical Ordと
同名binder・不正Coreの位置、既存の保存比較・user methodのstageを保ちます。
新しいinfix経路のprogram WASM受入は追加しません。

List prefixの共有はloweringの`--test match_decision`、backendの
`--test native list_prefix`、CLIの`--test rust_backend
list_prefix_decisions_and_effects_match_raw_strict_ts_and_native`で確認します。
元の生成TS両profileのstrictとfresh Bun/Node/nativeを比較し、guard失敗・長さ・
nested payload・alias・tail捕捉・Ref cycle・Failure解放を保ちます。lexicalな
`List` binderと未証明rootは認定しません。幅1/4/16/32はtrackedな
`measure_list_prefix.py`で測り、targetごとに改善と速度回帰を区別します。

閉じた標準tuple Show/Debug辞書のlazy reuseは、backendの`--test closed_display`と
CLIの`--test rust_backend closed_display_`で確認します。canonical owner/type/child/
arityとfactoryの純粋性を共有CoreIrで認定し、Parameter/user/generic/HKT/未証明factoryは
元の経路を保ちます。raw TS両profileのstrict、fresh Bun/Node/native、first read・
zero-step・別module・changing input・cache解放、helper名shadow・source map・reachability、
Ref/cold/Failure/Defect/finalizerを確認します。`measure_display_runtime.py`の通常bodyを
使うtimingと、別runのcold/warm allocator計数を分けます。両最適化の最終統合では、
全体gateとcanonical compiler WASMの再生成も必要です。

| 変更範囲 | コマンド | 含まれる検証 |
| --- | --- | --- |
| sample metadata / source | `bun run check:sample` | native CLI sample実行、全sampleのWASM compile / format、Playground sample manifestのfreshness |
| sample / Playground UI | `bun run check:playground` | sample基盤、Playground / catalog lint、Tour manifest、Playground test、TypeScript typecheck、Vite build |
| Web UI browser review | `cd apps/playground && bun run test:visual` | Chromiumで全HTML sampleのviewport / interaction / image fallback / Code / Explorerを確認し、review PNGとHTML reportを出力 |
| Rust / compiler / LSP workspace | `bun run check:rust` | Rust format、workspace test（対象crateだけなら `bun run check:rust -- -p <crate>`） |
| production browser dogfood | `bun run check:production:browser` | canonical CLIでWeb starter / Flowをdevelopment/release buildし、Chromium操作・text/input値の一致、source-map defaults、JS例外を検証。Playground依存とChromiumの事前installが必要。full/release source gateにも含む |
| production artifacts | `bun run check:production` | first-party 6 fixtureのrelease build、manifest shape、必要runtime、size budget、digest / identity / 再build、process実行。full / release source gateにも含む |
| conformance fixture | `bun run check:conformance` | canonical conformance runner（対象rootを引数で限定可能） |
| Portable .d.ts converter | `bun run check:dts:wasm` | filesystem featureを無効にしたconverterをwasm32-unknown-unknownへbuildし、host importなしで変換・metadata・再生成report・診断を実行。WASI SDK 25.0とRust WASM targetが必要 |
| compiler/runtime/WASM boundary | `bun run check:wasm` | committed Playground WASMの再生成と差分確認。portable declaration parser用のWASI SDK 25.0が必要 |
| pure program WASM / typed callback | `cargo test -p seseragi-cli --test program_wasm` | 実program WASMをBun／Nodeで実行し、TS両profile／native対照、generated型宣言のstrict、標準package module/asset buildと失敗時の保持、生成virtual packageのowner、path依存・初期化、callback／instance寿命とtrap後のretireを検証。`rustup target add wasm32-unknown-unknown`が必要。workspace full gateでも省略しない |
| providerless Effect program WASM | `cargo test -p seseragi-backend-rust --test program_wasm_effect --test program_wasm_effect_admission --test program_wasm_private_ref` と上記CLI lane | checked型・source stage・式のadmissionを確認し、実WASMのcold実行・内部data Ref・型付き失敗・明示Defect・取消・masked finalizer・非同期dispose・返却callbackをfresh Bun／Nodeで実行する。pure-v1と別ABIで、host providerやtrap後unwindの保証を加えない |
| typed host Effect program WASM | `cargo test -p seseragi-backend-rust --test program_wasm_host_effect --test program_wasm_host_ref` と `cargo test -p seseragi-cli --test program_wasm_host_effect` | closed unary host callback、内部data Refのcapture、通常TS／nativeとのbody比較、実WASMのFailure／Defect・遅い応答後の取消・masked LIFO release・disposeとhost lease解放、標準packageの元のtyped bridgeを確認。一般provider ABIやtrap後unwindは含めない |
| pure program WASM package watch | `cargo test -p seseragi-cli --test dev` と `bun test ./runtime/ts/tests/program-lifecycle.test.ts` | 計算moduleの外側path依存とUIの再build、失敗時の完成site／version保持、元のdev lifecycle／authority検査を確認。実WASMの保存callback拒否・旧root0・遅いloadの回収をfresh Bun／Nodeでも実行。既存のTSのみのdev期限を延ばさない |
| VS Code extension | `bun run check:extension` | official ID / legacy migration boundary、extension lint・test、host向け正式VSIXのarchive mode・展開・`--version-json` smoke、非LSP migration VSIXのpackage / verify |
| Local Web product E2E | native archiveとVSIXをpackage後に`bun run test:local-web-e2e` | 展開したCLIとinstall済みVSIXだけを使い、canonical multi-module Web projectのLSP / Dev / browser / edit / diagnostic / recovery / production buildを同一sourceで確認し、reportとPNGを`target/local-web-product-e2e`へ保存 |
| release metadata / native artifact | `bun run check:release` | canonical version source、Cargo/JS/WASM version同期、CHANGELOG、host向けCLI / LSP archive・checksum・再展開実行smoke |
| tag release source gate | `bun run check:release-gate` | repository-wide full gate相当。ただしnative / VSIXのpackage smokeは同一SHAを使うrelease matrix jobへ委譲し、成果物を二重生成しない |
| repository-wide | `bun run check` または `bun run check:full` | format、lint、Rust workspace、全conformance、native samples、WASM、Playground、extension |

devの停止要求とHTTP request待ち行列はCLIの`--bin seseragi dev::tests::`で、
実際の待受socketへ複数requestをqueueして確認します。停止前・handler中の停止では
次のrequest処理とwatch/buildへ進まず、通常時はchange pollを続けます。
`--test dev`の元13入力・recovery assertion・5秒のgraceful shutdown制限も保持します。
CLIだけの修復なのでcompiler WASMは変更せず、配布時のcanonical source gateで全体を確認します。

相互再帰するclosed EffectのTS注釈は、`cargo test -p seseragi-driver --test mutual_effect_returns`
と`cargo test -p seseragi-cli --test mutual_effect_returns`で確認します。元の生成TSを両profileでstrict検査し、
cold再実行・typed Failure・Defect後のscope解放をBun／Nodeとnativeで比較します。
共有Core解析を変える場合はlowering／driver crateの検証と、統合前の全体gateも必要です。

ConsoleのString契約はdriver／backendの`--test console_string_contract`とCLIの同名test、
`bun test runtime/ts/tests/console-string-contract.test.ts`で確認します。prelude／import／
保存したcallableの引数型、canonical ownerとscoped binder、raw生成TS両profileのstrict、
fresh Bun／Nodeとnativeの出力、cold再実行、引数評価時のDefectとscope解放を保ちます。
共有型互換性を変える場合はsemantics／driverの既存検証と最終統合gateも必要です。

HKTのADT payload・struct field・public aliasはdriverとCLIの
`--test hkt_data_projection`で確認します。元Issueの単独source、opaque aliasのfactory実値、
facadeと保存関数、元の生成TS両profileのstrict、fresh Bun／Nodeとnativeを比較します。
constructor binderを宣言する通常functionは同じchecked witnessを引数と返り値へ渡し、
制約付きfunctionの辞書段階と、別scopeの同名value binderを維持します。
外側のtype parameter名を内側で再使用する、仕様で禁止された宣言は位置付きで拒否します。
private fieldと偽造、別ownerやpayload違いの拒否を保ち、不正kindの位置付き診断は
driverの`--test hkt_annotation_kinds`でも確認します。一般のUnknown targetやHKT逆推論は
このlaneの受入へ追加しません。

標準primitive EqのTS投影は、`cargo test -p seseragi-cli --test string_equality`
と`--test inherited_equality`で確認します。異なるliteral・保存したmethod・部分適用を
元の生成TS両profileでstrict検査し、fresh Bun／Nodeで実行します。Int／String／Charの
native対照と、直接method・部分適用のsource一回評価・順序も保ちます。
Bool／UnitのEqと前置`!`のnative契約は、backend／CLIの`--test basic_scalars`と
上記の元literal fixtureで確認します。Unitも二つのsource引数を評価し、保存した部分適用、
別nominal ownerとscoped binderの拒否、Ref・cold再実行・Failure／Defect後の解放を保ちます。
前置operatorは関数値として参照できません。標準Float Eqは提供しません。
共有lowering／driverの検証に加え、統合前は全体gateとcanonical compiler WASMの検証が必要です。

native Clock sleepの有限契約は、backend／CLIの`--test clock_sleep`とbackend runtimeの
`clock_tests`で確認します。明示native CLI provider選択、canonical DurationとClockの
owner／kind／source引数、target receipt、通常path packageのTS用lock保持、元のTS両profileの
raw strict、fresh Bun／Nodeとnativeの結果を比較します。pending sibling、cold再実行、
typed Failure／Defect／取消、acquisitionのatomic登録とsleepを使うmasked finalizer、旧outputと入力保持、
timer／managed root解放を検証します。Clock now、WASMのClock row、一般provider ABIと
target別manifest／native provider lockはこのsliceに含めません。共通native schedulerを
変えるため、既存filesystem／Fiberとprogram WASMの回帰も確認し、統合時は全体gateを行います。

importしたADTのpayload identityと注釈付きEq部分適用は、driverの
`--test frontend_payload_identity --test frontend_inference --test imported_returns`で確認します。
constructorを公開するopaque ADT、local alias、同canonicalのdirect／facade／namespace import、
別ownerと不正な残り関数型の拒否を確認し、表示名でownerを推測しません。
CLIの`--test string_equality`と`--test nominal_lambda_identity`では、元の生成TS両profileの
strict、fresh Bun／Node、native、source引数の一回評価、cold再実行、Failure／Defect後の
finalizerとroot解放を比較します。FloatConversionErrorを含むnative fixtureには、
同errorのcanonical変換ABIが必要です。無注釈の演算子部分適用全般はこの修復に含めません。

generic implの演算子は、driverとCLIの`--test generic_impl_operators`で確認します。
生成instanceが元implのscoped binderを引き継ぎ、明示instance、別owner、importとreexport、
制約、overlapとfunctional dependencyの位置付き拒否を保つことを検証します。
元のTS両profileのstrict、fresh Bun／Nodeとnativeの結果、保存した演算子のsource一回評価、
Ref cycleとcold Effectの再実行、Failure／Defect後のfinalizerとroot解放も比較します。
一般aliasのhead mappingや、返却関数を続けて適用する未対応全般は含めません。

`do`末尾の分岐を含むcold Effectは、driverとCLIの
`--test deferred_effect_environment`で確認します。先に実行するstatementの環境・失敗型、
最終実行が返すEffect値の型、未実行のcold値を区別し、環境・成功型・失敗型の不正な
注釈を位置付きで拒否します。元のTS両profileのstrict、fresh Bun／Nodeとnativeで
再実行、先行処理の一回実行、Failure／Defect後のfinalizerとRef root解放を比較します。
共有frontendの修復なので、統合時は全体gateとcanonical compiler WASMの検証も必要です。

固定済みalias引数のconstructor sectionは、driverとCLIの
`--test alias_constructor_sections`で確認します。canonical ownerと宣言kindを根拠に、
残るslotが順序どおり一度だけ現れる場合に限って、並替・重複・消失済みprefixを投影します。
opaque providerのfacade、別owner・不正kind・未証明headの位置付き拒否、元のTS両profileの
strict、fresh Bun／Node、native、選択済みevidenceとsource一回評価、cold Effectと
Failure／Defect後の解放を比較します。未束縛の一般type lambdaやHKT逆推論は含めません。
共有frontendの変更なので、統合時は全体gateとcanonical compiler WASMの検証も必要です。

transparent aliasの残るvalue-kind parameterを並替・重複・破棄するconstructorは、
semanticsの`type_functions::tests`、loweringの`core::types::tests`、driverとbackendの
`--test alias_type_functions`、driver／CLIの`--test alias_constructor_sections`で確認します。
`Flip<A,B> = Pair<B,A>`、`Repeat<A> = Pair<A,A>`、`Erase<A> = Int`の明示type引数を使い、
scoped slotのcaptureを避けた適用、canonical owner／provider、alpha同値の重複診断、
renamed facade packageと保存methodのsource arityを保ちます。元のTS両profileのstrict、
fresh Bun／Nodeとnativeの出力・source一回評価、Ref・cold再実行・Failure／Defect後の
finalizerとroot解放を比較します。使わない束縛parameterもconstructorのarityに数え、
失われるfree instance parameter、不正owner／kind、未証明の選択は位置付きで拒否します。
このproducerはcanonical nominal application、scoped value slotと、各要素がvalue kindを
持つfunction・closed record・tupleを扱います。構造型aliasはdriverの
`--test structural_aliases --test structural_alias_facades`とCLIの
`--test alias_constructor_sections`で、renamed／namespace facade、返却method、元のTSの
strictと公開型consumerの負例、callbackのRef保存・cold再実行・Failure／Defect後の
finalizerとroot解放を確認します。共有semantics／lowering／driver全体の回帰も確認します。
CLIの`--test rust_backend alias_head`で元のfunction／record拒否入力がsourceを変えずに
emit／buildできることと、不正method型・失われるinstance parameterの位置付き拒否を確認します。
非対応という旧期待値を残さず、非実行・出力なし・source保持の既存assertionを保ちます。
row／requirement merge constructor、一般HKT逆推論、完全なtype-function同値判定と
program WASMの受入拡張は含めません。共有型表現・投影の変更なので、統合前は必要な
全体gateとcanonical compiler WASMの再生成・公開consumer検証も必要です。

制約付き関数やprimary辞書のmethodが返す関数の連続適用は、backendの
`--test returned_evidence_stages`とCLIの`--test alias_constructor_sections`で確認します。
宣言のsource境界・完全な返却型・owner／kind・辞書を検証し、各本体を次のsource引数より先に
実行します。保存・未使用部分適用、同名Unit／Int binder、Show aliasとUnit callback、
最初の段階のDefect、構造型alias経由のRef・cold再実行・Failure／Defect後の解放を比較します。
不正Core入力は位置付きで拒否し、program WASMの受入範囲は広げません。

各laneは `scripts/check-scoped.sh` を共通runnerとして使います。Playgroundのtest、
typecheck、buildはcatalog / Tour manifest確認をlane内で一度だけ済ませてから直接実行し、
`package.json`の個別scriptが同じcatalog確認を三回繰り返す構造を避けています。

`check:sample`はCLI実行対象だけでなくbrowser-interactive sampleもcommitted WASMで
project compileし、全sourceがformatterのcanonical outputと一致することを確認します。
manifest/hashだけが更新され、壊れたinteractive sourceが通過する状態にはしません。

sample・Tourのコマンド検証は、processの終了とstdout・stderrのEOFを両方確認します。
期限切れは独立した失敗として返し、後始末で再びstdioを無期限に待ちません。
POSIXでは親が先に終了しても所有process groupを停止します。親終了後の出力、
非zero終了、Unicode、期限と後始末は`scripts/sample-cli-process.test.ts`で確認します。

配布archiveの検証は、展開したCLI/LSPとarchive commandのprocess終了・stdioのcloseを
確認してから一時directoryを削除します。一時的な`EBUSY`等で失敗した場合も、削除を試みるのは
最初の試行を含めて最大6回です。再試行の待機は合計1500msまでです。
検証とcleanupが両方失敗した場合は両方のエラーを保持し、
検証が成功してもcleanupが失敗すれば失敗です。`bun test scripts/native-release.test.ts`で
正常・非zero・検証拒否・cleanup失敗と子process終了後の実ファイル削除を確認します。
cleanup失敗時は、実行したcommandのPID・終了code・exit/closeの観測も残します。
Windows固有のhandle競合の確認は、Windows CIと正式Releaseの実archive検証も必要です。

PostgreSQL Provider bundleの鮮度確認は、実際に解決された`node_modules`のsection comment
だけを正規化します。`bun test scripts/postgres-provider-bundle.test.ts`で通常installと
深いworktreeのsymlink installの生成bytes一致、実行コード変更時の拒否を確認します。

Playgroundのruntime resolverは`runtime/ts/package.json`のexportsと
`runtime/providers/package.json`の`runtime-browser/*` exportsから生成します。
`bun run --cwd apps/playground runtime:generate`で更新し、`runtime:check`と
Playground testsが生成物・compiler Runtime ABIのmodule/export・browser provider
manifestとのparityを検証します。host adapterはruntime packageの`browser` condition、
native専用moduleは`browser: null`で宣言し、Playgroundへ別のmodule一覧を足しません。
providerの`browser/` sourceから`runtime-browser/` packageへのprojectionはRustの
package staging testでも同一内容を検証します。

## Standard instance audit

Prelude auditの`specified-but-implementation-missing`行は、canonical registryの
`tracking_issue`に正のIssue番号を必ず持ちます。surface生成時に検証するため、
unit testだけでなくconformanceのartifact照合も未追跡行を拒否します。
GitHubへの問い合わせは不要です。実装完了時はmissing行を削除し、implemented /
structural / intentionally unavailable行に古いtracking metadataを残しません。

`prelude::surface`のsynthetic missing testは、現在のmissing一覧が空でもこの契約と
JSONの`trackingIssue`投影を検証します。auditだけの変更はこのsemantics testと
conformanceの`stdlib_surface` testを先に実行できます。

## Dependency installation

検証用のBunはCIとlocalで **1.4.4** に揃える。versionの基準はrootの
`.bun-version`で、GitHub Actionsも同じfileを読む。localでは`bun --version`と
`cat .bun-version`が一致することを確認してから検証する。

以前の1.3.9ではLinux x64 WASMのIPIntクラッシュと同じstackを全体gateで観測し、
[WebKit更新 #26922](https://github.com/oven-sh/bun/pull/26922)を含む1.3.11へ
揃えていた。1.4.4でBrotli decoderのCVE-2026-111497が修正されたため更新し、
この境界の実行も省略しない。
WASMの実行・assertionを省略したり、JSCの実行tierを無効化したりはしない。
既存Docsの過去の実測version表記は、その検証時点の記録として残す。

通常のscoped checkは依存installを行わず、lockfileで管理されたlocal binaryだけを使います。
必要なdependencyが無い場合は暗黙downloadへ進まず、bootstrap commandを表示して失敗します。
新しいworktreeやCI runnerでは、検証前に対象workspaceを一度bootstrapしてください。

```sh
bun install --frozen-lockfile
cd apps/playground && bun install --frozen-lockfile
cd ../../extensions/seseragi && bun install --frozen-lockfile
```

`check:sample`と`check:playground`はcommitted WASMまたは固定されたPlayground
TypeScript toolchainを使うため、Playground dependencyのbootstrapが必要です。
`check:conformance`はそれらに加えてruntime package probeがroot側のNode型を使うため、
rootとPlaygroundの両方をbootstrapしてください。lockfileまたは依存関係を変更した場合も、
該当workspaceでfrozen installを明示的に行います。full gateはrootとPlaygroundを先に
bootstrapし、extension packaging時にもfrozen installを行います。

## Browser executable for Playground Bun tests

PlaygroundのBun browser testsは、既定でlockfileのPlaywrightに対応するChromiumを使います。
Cloudなどでインストール済みGoogle Chromeを使う場合は、その実行ファイルの絶対pathを
`SESERAGI_TEST_BROWSER_PATH`へ設定します。起動したbrowserのpathとversionをログへ残し、
指定pathで起動できない場合はそのまま失敗します。この設定はBun tests用で、
`test:visual`やproduction browser scriptsのbrowser選択は変更しません。

```sh
cd apps/playground
SESERAGI_TEST_BROWSER_PATH=/path/to/google/chrome bun test tests
```

## Web UI browser review

`apps/playground/tests/fixtures/web-ui-regression.json`が、全HTML sample、320px /
iPhone / Android / landscape / desktop、必要surface、interaction stateを定義します。
通常の`check:playground`はこのmatrixとsource readabilityをBun testで固定し、Chromiumを
暗黙downloadしません。実ブラウザを使うreview時だけ次を実行します。

```sh
cd apps/playground
bun run test:visual:install
bun run test:visual
```

Playwrightはfixed Unsplash URLをdeterministicなlocal SVGへrouteし、layout、overflow、
contrast、keyboard reachable control、image failure fallbackを確認します。代表stateは
review済みscreenshot baselineとも比較し、失敗時は`test-results/web-ui-review/`へ
expected / actual / diff、trace、HTML reportを残します。OS固有のfont rasterizationを
layout差分と誤認しないようbaselineはplatform別に保持し、localとGitHub Actionsはともに
`bun run test:visual`をverify commandとして使います。

full-page baselineはPlayground chromeとPreview / Editor / Explorerの位置関係を確認するため
維持します。ただし許容差分は画像全体の比率ではなく、各baselineが監視するPreview body、
Editor panel、Workspace shellの面積の1%をpixel数へ変換して判定します。色差threshold 0.25は
platform別font rasterizationを吸収し、局所surfaceのspacing / typography / alignment差分が
大きな余白へ薄まらないようにします。感度testは三種類の意図的な局所差分がこのbudgetを
超えて比較失敗になることを固定します。

意図したUI変更でbaselineを更新するときは、先に
[`SHOWCASE_QUALITY.md`](./SHOWCASE_QUALITY.md)のhuman reviewを完了し、review理由を必須にした
次のコマンドを使います。PNGと承認済み`showcase-review.json`のSHA-256、理由は
`e2e/visual-baselines.review.json`へ記録され、PNGだけまたはapproval artifactだけの変更は
verify前に失敗します。

```sh
bun run test:visual:update -- "変更理由"
```

update commandはreview対象のbaselineを持つWeb UI regression specだけを再実行してPNGと
review hashを更新します。通常の`bun run test:visual`はguideを含む全Playwright specを
引き続き実行するため、baseline更新のたびに無関係なbrowser testを重ねず、最終verifyの
coverageは狭めません。

Linux baselineも同じupdate commandを使い、`Web UI visual regression`の
workflow_dispatchへ`update_reason`を渡して生成します。

## Full gateを実行する条件

次のいずれかに該当するときだけ、scoped laneに加えて `bun run check` を実行します。

- compiler / runtime / WASM基盤を変更した
- 複数の領域を横断した
- release前の統合確認を行う
- #201のqueueをmainへ統合する
- Issue本文がfull checkを明示的に要求する

full gateを実行した場合は、作業ログまたはPR本文へ「必要理由」と「結果」を記録します。
長時間無出力になった場合は、該当phaseとprocess activityを確認し、放置しません。

## CIとlocalの責務

- localの末端Issue作業は、変更範囲に対応するscoped laneを担当する。
- `.github/workflows/vscode-extension.yml` はextension path変更時のOS別VSIX package、
  archive mode、展開後native LSPの`--version-json` smokeを担当する。local
  `check:extension`はhost platformの短い再現確認であり、tag releaseでも同じ検証器を
  upload前のrelease VSIXへ再実行する。
- `.github/workflows/release-contract.yml` はversion sourceやartifact namingの変更で
  `check:release`を実行し、Linux native archiveも実binaryで検証する。tag releaseは
  `v<version>`だけを受け付け、tag commitのmain包含と`check:release-gate`成功後にだけ
  同一SHAの全platform artifactを生成し、upload前と再download後に検証する。
- repository-wideのfull gateは、compiler/runtime/WASM変更、release、queue統合などの
  integration pointで明示的に実行する。CIに同じ長時間検査を各sample変更へ無条件に重ねない。
- scoped laneが不足している場合は、対象コマンドを個別に実行して結果を記録し、lane追加を
  次の基盤Issueとして扱う。

### Release profile implementation checks

`cargo test -p seseragi-cli --test profiles` compiles the canonical performance
fixtures in both profiles and executes each in a fresh process. It also runs
`scripts/release-shapes.ts` against the emitted release module and fixture
predicates. Install root dependencies with `bun install --frozen-lockfile` first.
The checker resolves canonical module symbols through generated inspection roots;
metadata does not report optimization success. Its AST proof is conservative:
unsupported wrapper/projection shapes fail instead of being guessed. Inspection
roots are retained in release output; bundling/DCE is a separate boundary.

`bun test scripts/release-shapes.test.ts runtime/ts/tests/effect-stack.test.ts`
checks shape rejection cases and 100,000 Effect binds independently of pure TCO.
The conformance lane includes these checks. Profile/compiler/runtime integration
still requires the full gate and regenerated WASM during Promotion.

WASM再生成時は[WASI SDK 25.0](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-25)のホストOS/CPU用archiveを展開し、`WASI_SDK_PATH`をそのdirectoryへ設定する。C parserのcompilerも固定し、Rust/wasm-packだけが同じでも発生する生成物差分を防ぐ。

WASM glueには`cargo install wasm-bindgen-cli --version 0.2.126 --locked`でCargo.lockと同じCLIを用意する。`wasm-pack`の自動install fallbackはCLI内部の依存versionを変えるため、buildは`--mode no-install`で既存のCLIを使う。

配布用WASMの正規生成hostはLinux x64。`check:wasm`は生成物全体のbyte一致を検証し、PRとrelease promotionのLinux gateで必須とする。Cargoはhostで実行するbuild script/proc macroのmetadataを依存先へ伝播するため、macOS生成物とはcode/dataを含む差分が生じ得る。macOS CIでは同じ固定toolchainと2つの独立したfresh Cargo targetで再構築する。1回目のfresh outputと2回目のrepo/pkg stagingで全packageのfile集合とbyte一致を確認し、変換・workspace・browser回帰を検証する。さらに両hostの変換JSON出力をCIで完全一致比較し、macOS生成物で配布artifactを置き換えない。

captureしたprimitiveの推論identityは、`cargo test -p seseragi-semantics --lib`、
`cargo test -p seseragi-driver --lib --test compile_module --test captured_primitive_identity`、
`cargo test -p seseragi-cli --test captured_primitive_identity`で確認する。
同名nominalの拒否とscoped evidenceを保ち、元のsourceとcold Effectの正常・Failure・Defectを
raw TS両profileのstrict、fresh Bun/Node、nativeとfinalizer・managed rootで比較する。
標準moduleを使うfixtureはstandaloneのsemantics helperへ渡さず、共有driver経由で検証する。
共有推論の変更には、統合前のfull gateと正規WASMの更新も必要となる。

nativeのString Eqは、`cargo test --locked -p seseragi-backend-rust --test string_equality --test equality_partials`と
`cargo test --locked -p seseragi-cli --test string_equality`で確認する。canonicalなEqのowner・型・evidenceを
変えた負例を拒否し、Unicode/NUL・保存method・部分適用・cold再実行・Failure/Defect・finalizer/root0を
raw strict TS両profile、fresh Bun/Nodeとnativeで比較する。file条件分岐では入力と失敗時の既存出力も保つ。
共有runtimeを使うため、通常Web buildのprogram WASMと元のtyped bridge、callback/instanceの解放も検証する。
左右のsource helperだけを計測するtraceで、一回評価と順序を比較し、生成Eq式は変更しない。
共有runtimeの統合にはfull gateと正規compiler WASMの更新も必要となる。

nativeのtext.lines/joinは、`cargo test --locked -p seseragi-backend-rust --lib text_tests`、
`cargo test --locked -p seseragi-backend-rust --test text_processing`と
`cargo test --locked -p seseragi-cli --test text_processing`で確認する。Unicode改行・CRLF・空入力と
残る文字列のbytes、保存joinの一回評価・順序、cold再実行・Failure/Defect・finalizer/root0を
raw strict TS両profile・fresh Bun/Node・nativeで比較する。fileの入力と読込失敗時の出力を保護し、
通常Web buildのprogram WASM、元typed bridgeのstrict・skipLibCheck:false、callback/instance解放も
確認する。公開Array codecや一般provider ABIはこの受入に含めない。共有runtimeの統合には
full gateと正規compiler WASMの更新が必要となる。

nativeの十進Int parseは、`cargo test --locked -p seseragi-backend-rust --lib int_parse_tests`、
`cargo test --locked -p seseragi-backend-rust --test int_parse`と
`cargo test --locked -p seseragi-cli --test int_parse`で確認する。標準のIntParseErrorのowner・kind・
source arity・evidenceを検証し、fileとpath packageの集計、入力と失敗時の既存出力の保持、
保存parseの一回評価、cold再実行・Failure/Defect・finalizer/root0をraw strict TS両profile・
fresh Bun/Node・nativeで比較する。標準IntParseErrorのShowはdriver/backendの
`--test int_parse_error_show`とCLIの`--test int_parse standard_parse_error_show`で確認する。
4つのconstructor、同名local型とpath package、元のraw strict TS両profile、cold失敗と
Defect/finalizer、fileの既存出力保持を比較する。radix指定、他の数値変換API、一般program WASM、
IntParseErrorのEq/Debugはこの受入に含めない。共有runtimeの統合にはfull gateと
正規compiler WASMの更新が必要となる。

nativeのFloat parseは、`cargo test --locked -p seseragi-backend-rust --lib parse_tests`、
`cargo test --locked -p seseragi-backend-rust --test float_parse --test int_parse --test int_parse_error_show`と
`cargo test --locked -p seseragi-cli --test float_parse --test int_parse`で確認する。
共通number parse ABIのowner・kind・source arity・evidence検査と既存Int/Showの拒否条件を保持し、
decimal/exponent構文、UTF-8 byte offset、構文失敗とoverflowの順序、subnormal・underflow・
signed zero・明示NaN/Infinityを確認する。sourceとpath packageで小数の各行を集計し、
元のraw strict TS両profile・fresh Bun/Node・nativeで比較する。入力と失敗時の既存出力、
保存parseの一回評価、cold再実行・Failure/Defect・finalizer/root0を検証する。
FloatParseErrorの新しいShow/Debug/Eq、他の数値変換API、一般program WASMはこの受入に含めない。
共有backend/runtimeの統合にはfull gateと正規compiler WASMの更新が必要となる。

nativeのFloat丸め・Int変換は、backendの`--lib float_conversion_tests`と
`--test float_conversion`、CLIの`--test float_conversion`で確認する。
canonicalなRoundingModeとFloatConversionErrorのowner/kind・source arity・evidenceを保ち、
6modeの中間値と隣接値、signed zero、safe Int境界と非finite失敗を比較する。
source/path packageの元生成TS両profileをstrict検査し、fresh Bun/Node/nativeで
入力と失敗時の旧出力、一回評価と保存・返り値converter、cold Failure/Defect・finalizer/root0を確認する。
program WASMは位置付き拒否を保ち、Float Eq/Ord・Pow/stdmathや新しいerror Showを含めない。
共有numeric ABIの変更には元Int/Float parseとShowの回帰確認、統合前の全体gateと正規compiler WASMも必要となる。

nativeのInt剰余は、`cargo test --locked -p seseragi-backend-rust --test int_remainder --test int_division --test operator_partials`と
`cargo test --locked -p seseragi-cli --test int_remainder`で確認する。標準Remのowner・kind・source arity・
選択済みevidenceを検証し、既存Divとprogram WASMの拒否条件を保持する。正負の値とsafe integer境界、
直接呼出し・保存method・generic/返り値関数の一回評価と順序、cold再実行、zero divisorのDefectと
finalizer/root0を、元のraw strict TS両profile・fresh Bun/Node・nativeで比較する。
sourceと凍結したpath packageで各行の整数の剰余を別fileへ書き、入力と失敗時の既存出力を保持する。
Pow、新しいchecked remainder API、一般program WASMへの剰余の追加は受入に含めない。
共有backend/runtimeの統合にはfull gateと正規compiler WASMの更新が必要となる。

nativeのInt/Float累乗は、backendの`--lib power`と`--test native_power`、CLIの
`--test native_power`で確認する。標準Powと`std/float.power`のcanonical owner/kind、
辞書、source arity、保存methodと返り値関数、同名のlexical binderを区別する。
Intのsafe integer境界・負指数とoverflowのDefect、FloatのNaN/Infinity/signed zeroと
指数の偶奇を、元生成TS両profileのstrict検査とfresh Bun/Node/nativeで比較する。
直接・保存した演算子のsource一回評価と順序、Refとcold再実行、型付きFailure/Defect後の
finalizer/root0、sourceと凍結したpath packageでの入力・失敗時の旧出力保持も検証する。
一般program WASMへの累乗の追加、残るstdmath API、Float Eq/Ordは含めない。
既存Div/Rem・Float parse/conversion・operator partialの回帰を確認し、共有backend/runtimeの
統合前は全体gateと正規compiler WASMの検証も必要となる。
