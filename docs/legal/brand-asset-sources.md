# 品牌图标来源核查

核查日期：2026-10-23。当前清单包含 204 个图像文件及 `src/icons/extracted/index.ts` 中 70 个内联 SVG。品牌目录中有 100 个文件；其余 104 个文件为截图、架构图、应用图标等，保留在机器清单中，不混算为独立品牌。

## 能确认的来源

- 导入根提交的 `package.json` 声明 `@lobehub/icons-static-svg`，`pnpm-lock.yaml` 锁定 **1.73.0**。同一提交的 `scripts/extract-icons.js` 明确从该包的 `icons` 目录复制 SVG。
- 本次下载该历史包并验证 SHA-512 与导入根锁文件一致。**49 个文件和 49 个内联 SVG 与归档成员逐字节一致**；二者有重复展示，不代表 98 个独立品牌。
- 已从 npm gitHead `798b90d7d65cd764b7a2dee1f95e7bd56eacb602` 补齐 Lobe Icons MIT 原文，加入 THIRD_PARTY_NOTICES 与 Tauri 资源。仅识别到库来源，不把软件许可证当作各品牌的商标授权证明。

[导入时提取脚本](https://github.com/KeaneFeng/cc-gateway-pro/blob/0f69cb085/scripts/extract-icons.js) · [历史锁文件](https://github.com/KeaneFeng/cc-gateway-pro/blob/0f69cb085/pnpm-lock.yaml) · [库原始许可](https://raw.githubusercontent.com/lobehub/lobe-icons/798b90d7d65cd764b7a2dee1f95e7bd56eacb602/LICENSE) · [原文及完整性值](asset-license-notices.json)

## 本轮新增的内容对应（非商标授权）

- `bailian.svg`：与 Lobe 1.73.0 `package/icons/bailian.svg` 仅差本地末尾一个换行；原始文件和当前文件 SHA-256 均保留，没有改写图形来制造匹配。
- `catcoder.svg` 及内联 `catcoder`：与完整性验证通过的 Lobe **1.98.0** `package/icons/kwaikat.svg` 内容匹配（只忽略外层空白），该包 gitHead 为 `86b3f06ad9a4cc57011a56e5a7d3a5d4f1e8d835`，MIT 原文已补入声明集合。这确立库内对应素材及许可，不证明仓库当初从该版本下载，也不证明该素材被用作 `catcoder` 品牌标识已获授权。
- 原 49 文件/49 内联的逐字节证据不变；新增 2 文件/1 内联单独标注匹配方式。两种证据不可混称为全部逐字节一致。

## 不能确认的范围与原因

历史复核中有 51 个品牌文件和 21 个内联 SVG 未逐字节匹配；本轮补证后，仍有 **49 个品牌文件和 20 个内联 SVG** 未建立库/供应商内容对应。下表分别记录保留自导入、导入后修改、后续新增三类情况。Git 提交能证明进入本仓库的过程；它不能单独证明原网站地址或品牌方允许再分发。名称相同、视觉相似、SVG 内出现 lobe 前缀，都没有被用作精确来源证明。

没有删除、更换或重画现有品牌图标。需要补充供应商原始 URL、对应版本、作者/许可或品牌使用条款，才能进一步确认。截图和本项目绘制的图需另行确认素材权属；不因根 LICENSE 为 MIT 就自动给第三方品牌授权。

## 品牌文件逐项结果

| 文件                                            | 来源结果             | 具体证据 / 待确认原因                                                                                                                                                                                |
| ----------------------------------------------- | -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/icons/extracted/ClaudeApi.png`             | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/TeamoRouter-icon-dark.png` | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/a6-icon.png`               | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/a4db787ab0fe6f27b7957da048d9440b55735996) |
| `src/icons/extracted/aicodemirror.svg`          | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/aicoding.svg`              | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/aihubmix-color.svg`        | Lobe 1.73.0 精确匹配 | 归档 `package/icons/aihubmix-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)      |
| `src/icons/extracted/algocode.svg`              | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/alibaba.svg`               | Lobe 1.73.0 精确匹配 | 归档 `package/icons/alibaba-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)       |
| `src/icons/extracted/amuxapi-icon.svg`          | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/anthropic.svg`             | Lobe 1.73.0 精确匹配 | 归档 `package/icons/anthropic.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)           |
| `src/icons/extracted/apikeyfun.png`             | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/apinebula_icon.png`        | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/atlascloud_icon.png`       | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/aws.svg`                   | Lobe 1.73.0 精确匹配 | 归档 `package/icons/aws-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)           |
| `src/icons/extracted/azure.svg`                 | Lobe 1.73.0 精确匹配 | 归档 `package/icons/azure-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)         |
| `src/icons/extracted/baidu.svg`                 | Lobe 1.73.0 精确匹配 | 归档 `package/icons/baidu-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)         |
| `src/icons/extracted/bailian.svg`               | Lobe 1.73.0 内容对应 | 与 `package/icons/bailian.svg` 仅差末尾换行；MIT 已收集，商标权限另核。                                                                                                                              |
| `src/icons/extracted/bytedance.svg`             | Lobe 1.73.0 精确匹配 | 归档 `package/icons/bytedance-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)     |
| `src/icons/extracted/byteplus.png`              | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/catcoder.svg`              | Lobe 1.98.0 内容对应 | 对应 `package/icons/kwaikat.svg`；仅忽略外层空白；不证明 catcoder 品牌使用授权。                                                                                                                     |
| `src/icons/extracted/ccsub.svg`                 | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/chatglm.svg`               | Lobe 1.73.0 精确匹配 | 归档 `package/icons/chatglm-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)       |
| `src/icons/extracted/cherryin.png`              | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/claude.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/claude-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)        |
| `src/icons/extracted/claudecn.png`              | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/claw.svg`                  | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/cloudflare.svg`            | Lobe 1.73.0 精确匹配 | 归档 `package/icons/cloudflare-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)    |
| `src/icons/extracted/code0.png`                 | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/cohere.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/cohere-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)        |
| `src/icons/extracted/copilot.svg`               | Lobe 1.73.0 精确匹配 | 归档 `package/icons/copilot-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)       |
| `src/icons/extracted/crazyrouter.svg`           | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/ctok.svg`                  | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/cubence.svg`               | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/dds.svg`                   | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/deepseek.svg`              | Lobe 1.73.0 精确匹配 | 归档 `package/icons/deepseek-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)      |
| `src/icons/extracted/doubao.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/doubao-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)        |
| `src/icons/extracted/eflowcode.png`             | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/etok.png`                  | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/fenno-icon.webp`           | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/gemini.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/gemini-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)        |
| `src/icons/extracted/gemma.svg`                 | Lobe 1.73.0 精确匹配 | 归档 `package/icons/gemma-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)         |
| `src/icons/extracted/github.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/github.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)              |
| `src/icons/extracted/githubcopilot.svg`         | Lobe 1.73.0 精确匹配 | 归档 `package/icons/githubcopilot.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)       |
| `src/icons/extracted/google.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/google-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)        |
| `src/icons/extracted/googlecloud.svg`           | Lobe 1.73.0 精确匹配 | 归档 `package/icons/googlecloud-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)   |
| `src/icons/extracted/grok.svg`                  | Lobe 1.73.0 精确匹配 | 归档 `package/icons/grok.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                |
| `src/icons/extracted/hermes.png`                | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/huawei.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/huawei-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)        |
| `src/icons/extracted/huggingface.svg`           | Lobe 1.73.0 精确匹配 | 归档 `package/icons/huggingface-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)   |
| `src/icons/extracted/hunyuan.svg`               | Lobe 1.73.0 精确匹配 | 归档 `package/icons/hunyuan-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)       |
| `src/icons/extracted/huoshan.png`               | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/kimi.svg`                  | Lobe 1.73.0 精确匹配 | 归档 `package/icons/kimi-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)          |
| `src/icons/extracted/lemondata.png`             | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/lioncc.svg`                | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/longcat-color.svg`         | Lobe 1.73.0 精确匹配 | 归档 `package/icons/longcat-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)       |
| `src/icons/extracted/mcp.svg`                   | Lobe 1.73.0 精确匹配 | 归档 `package/icons/mcp.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                 |
| `src/icons/extracted/meta.svg`                  | Lobe 1.73.0 精确匹配 | 归档 `package/icons/meta-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)          |
| `src/icons/extracted/micu.svg`                  | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/midjourney.svg`            | Lobe 1.73.0 精确匹配 | 归档 `package/icons/midjourney.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)          |
| `src/icons/extracted/minimax.svg`               | Lobe 1.73.0 精确匹配 | 归档 `package/icons/minimax-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)       |
| `src/icons/extracted/mistral.svg`               | Lobe 1.73.0 精确匹配 | 归档 `package/icons/mistral-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)       |
| `src/icons/extracted/modelscope-color.svg`      | Lobe 1.73.0 精确匹配 | 归档 `package/icons/modelscope-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)    |
| `src/icons/extracted/nekocode-icon.png`         | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/newapi.svg`                | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/notion.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/notion.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)              |
| `src/icons/extracted/novita.svg`                | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/nvidia.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/nvidia-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)        |
| `src/icons/extracted/ollama.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/ollama.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)              |
| `src/icons/extracted/openai.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/openai.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)              |
| `src/icons/extracted/opencode-logo-light.svg`   | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/openrouter.svg`            | Lobe 1.73.0 精确匹配 | 归档 `package/icons/openrouter.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)          |
| `src/icons/extracted/packycode.svg`             | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/palm.svg`                  | Lobe 1.73.0 精确匹配 | 归档 `package/icons/palm-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)          |
| `src/icons/extracted/pateway.jpg`               | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/perplexity.svg`            | Lobe 1.73.0 精确匹配 | 归档 `package/icons/perplexity-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)    |
| `src/icons/extracted/pipellm.png`               | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/qiniu.png`                 | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/qwen.svg`                  | Lobe 1.73.0 精确匹配 | 归档 `package/icons/qwen-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)          |
| `src/icons/extracted/rc.svg`                    | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/relaxcode.png`             | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/runapi.jpg`                | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/shengsuanyun.svg`          | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/siliconflow.svg`           | Lobe 1.73.0 精确匹配 | 归档 `package/icons/siliconcloud-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)  |
| `src/icons/extracted/sssaicode.svg`             | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/stability.svg`             | Lobe 1.73.0 精确匹配 | 归档 `package/icons/stability-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)     |
| `src/icons/extracted/stepfun.svg`               | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/subrouter.svg`             | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/sudocode-us.png`           | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/sudocode.png`              | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/tencent.svg`               | Lobe 1.73.0 精确匹配 | 归档 `package/icons/tencent-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)       |
| `src/icons/extracted/ucloud.svg`                | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/unity2.png`                | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/vercel.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/vercel.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)              |
| `src/icons/extracted/wenxin.svg`                | Lobe 1.73.0 精确匹配 | 归档 `package/icons/wenxin-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)        |
| `src/icons/extracted/xai.svg`                   | Lobe 1.73.0 精确匹配 | 归档 `package/icons/xai.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                 |
| `src/icons/extracted/xiaomimimo.svg`            | 仅能确认仓库沿革     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)                     |
| `src/icons/extracted/yi.svg`                    | Lobe 1.73.0 精确匹配 | 归档 `package/icons/yi-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)            |
| `src/icons/extracted/zeroone.svg`               | Lobe 1.73.0 精确匹配 | 归档 `package/icons/zeroone.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)             |
| `src/icons/extracted/zetaapi-icon.png`          | 仅能确认仓库沿革     | 导入之后新增；能定位首次加入提交，但本次未找到可对应当前字节的原始供应商文件及单独授权。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/08204a91df685b5a5429b17f037a999db4e11b78) |
| `src/icons/extracted/zhipu.svg`                 | Lobe 1.73.0 精确匹配 | 归档 `package/icons/zhipu-color.svg`；MIT 原文已收集，品牌使用条件仍需分别确认。 [文件首次加入](https://github.com/KeaneFeng/cc-gateway-pro/commit/0f69cb08506238d89bae17d9fbf23a70eb92e25f)         |

## 内联 SVG 逐项结果

文件首次加入提交不是某个内联图标首次出现的证明；机器清单中的 `file_introduction_commit` 仅描述文件历史。下列匹配直接基于当前 SVG 字符串的 SHA-256。

| icons 键        | 结果                 | 证据 / 具体缺口                                                            |
| --------------- | -------------------- | -------------------------------------------------------------------------- |
| `aicodemirror`  | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `aicoding`      | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `aigocode`      | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `alibaba`       | Lobe 1.73.0 精确匹配 | `package/icons/alibaba-color.svg`                                          |
| `amux`          | 未确认最初素材源     | 导入时路径已存在，但当前内容已变化，不能把旧文件来源直接套用到现有图形。   |
| `anthropic`     | Lobe 1.73.0 精确匹配 | `package/icons/anthropic.svg`                                              |
| `aws`           | Lobe 1.73.0 精确匹配 | `package/icons/aws-color.svg`                                              |
| `azure`         | Lobe 1.73.0 精确匹配 | `package/icons/azure-color.svg`                                            |
| `baidu`         | Lobe 1.73.0 精确匹配 | `package/icons/baidu-color.svg`                                            |
| `bytedance`     | Lobe 1.73.0 精确匹配 | `package/icons/bytedance-color.svg`                                        |
| `chatglm`       | Lobe 1.73.0 精确匹配 | `package/icons/chatglm-color.svg`                                          |
| `claude`        | Lobe 1.73.0 精确匹配 | `package/icons/claude-color.svg`                                           |
| `cloudflare`    | Lobe 1.73.0 精确匹配 | `package/icons/cloudflare-color.svg`                                       |
| `cohere`        | Lobe 1.73.0 精确匹配 | `package/icons/cohere-color.svg`                                           |
| `copilot`       | Lobe 1.73.0 精确匹配 | `package/icons/copilot-color.svg`                                          |
| `crazyrouter`   | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `cubence`       | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `deepseek`      | Lobe 1.73.0 精确匹配 | `package/icons/deepseek-color.svg`                                         |
| `doubao`        | Lobe 1.73.0 精确匹配 | `package/icons/doubao-color.svg`                                           |
| `gemini`        | Lobe 1.73.0 精确匹配 | `package/icons/gemini-color.svg`                                           |
| `gemma`         | Lobe 1.73.0 精确匹配 | `package/icons/gemma-color.svg`                                            |
| `github`        | Lobe 1.73.0 精确匹配 | `package/icons/github.svg`                                                 |
| `githubcopilot` | Lobe 1.73.0 精确匹配 | `package/icons/githubcopilot.svg`                                          |
| `google`        | Lobe 1.73.0 精确匹配 | `package/icons/google-color.svg`                                           |
| `googlecloud`   | Lobe 1.73.0 精确匹配 | `package/icons/googlecloud-color.svg`                                      |
| `grok`          | Lobe 1.73.0 精确匹配 | `package/icons/grok.svg`                                                   |
| `huawei`        | Lobe 1.73.0 精确匹配 | `package/icons/huawei-color.svg`                                           |
| `huggingface`   | Lobe 1.73.0 精确匹配 | `package/icons/huggingface-color.svg`                                      |
| `hunyuan`       | Lobe 1.73.0 精确匹配 | `package/icons/hunyuan-color.svg`                                          |
| `kimi`          | 未确认最初素材源     | 导入时路径已存在，但当前内容已变化，不能把旧文件来源直接套用到现有图形。   |
| `meta`          | Lobe 1.73.0 精确匹配 | `package/icons/meta-color.svg`                                             |
| `midjourney`    | Lobe 1.73.0 精确匹配 | `package/icons/midjourney.svg`                                             |
| `minimax`       | Lobe 1.73.0 精确匹配 | `package/icons/minimax-color.svg`                                          |
| `zenmux`        | 未确认最初素材源     | 导入时路径已存在，但当前内容已变化，不能把旧文件来源直接套用到现有图形。   |
| `mistral`       | Lobe 1.73.0 精确匹配 | `package/icons/mistral-color.svg`                                          |
| `newapi`        | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `notion`        | Lobe 1.73.0 精确匹配 | `package/icons/notion.svg`                                                 |
| `ollama`        | Lobe 1.73.0 精确匹配 | `package/icons/ollama.svg`                                                 |
| `openai`        | Lobe 1.73.0 精确匹配 | `package/icons/openai.svg`                                                 |
| `openclaw`      | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `packycode`     | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `palm`          | Lobe 1.73.0 精确匹配 | `package/icons/palm-color.svg`                                             |
| `perplexity`    | Lobe 1.73.0 精确匹配 | `package/icons/perplexity-color.svg`                                       |
| `qwen`          | Lobe 1.73.0 精确匹配 | `package/icons/qwen-color.svg`                                             |
| `stability`     | Lobe 1.73.0 精确匹配 | `package/icons/stability-color.svg`                                        |
| `tencent`       | Lobe 1.73.0 精确匹配 | `package/icons/tencent-color.svg`                                          |
| `vercel`        | Lobe 1.73.0 精确匹配 | `package/icons/vercel.svg`                                                 |
| `wenxin`        | Lobe 1.73.0 精确匹配 | `package/icons/wenxin-color.svg`                                           |
| `xai`           | Lobe 1.73.0 精确匹配 | `package/icons/xai.svg`                                                    |
| `xiaomimimo`    | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `yi`            | Lobe 1.73.0 精确匹配 | `package/icons/yi-color.svg`                                               |
| `zeroone`       | Lobe 1.73.0 精确匹配 | `package/icons/zeroone.svg`                                                |
| `zhipu`         | Lobe 1.73.0 精确匹配 | `package/icons/zhipu-color.svg`                                            |
| `openrouter`    | Lobe 1.73.0 精确匹配 | `package/icons/openrouter.svg`                                             |
| `rc`            | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `lioncc`        | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `longcat`       | Lobe 1.73.0 精确匹配 | `package/icons/longcat-color.svg`                                          |
| `modelscope`    | Lobe 1.73.0 精确匹配 | `package/icons/modelscope-color.svg`                                       |
| `aihubmix`      | Lobe 1.73.0 精确匹配 | `package/icons/aihubmix-color.svg`                                         |
| `opencode`      | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `siliconflow`   | Lobe 1.73.0 精确匹配 | `package/icons/siliconcloud-color.svg`                                     |
| `micu`          | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `ucloud`        | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `sssaicode`     | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `stepfun`       | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `catcoder`      | Lobe 1.98.0 内容对应 | 对应 `package/icons/kwaikat.svg`；原 MIT 已收集，品牌名/图形关系仍需确认。 |
| `mcp`           | Lobe 1.73.0 精确匹配 | `package/icons/mcp.svg`                                                    |
| `novita`        | 未确认最初素材源     | 与导入版本逐字节一致，但导入记录没有证明最初的供应商来源及单独授权。       |
| `nvidia`        | Lobe 1.73.0 精确匹配 | `package/icons/nvidia-color.svg`                                           |
| `bailian`       | Lobe 1.73.0 精确匹配 | `package/icons/bailian.svg`                                                |

所有文件内容摘要、导入前后比较及具体源归档路径见 [asset-provenance.json](asset-provenance.json)。源归档的完整性校验不意味着对未知品牌权利作了保证。

## 对剩余素材的处理建议

1. **沿革明确但缺供应商证据**：逐项向相应品牌维护者取得原始 URL、文件摘要/版本和允许使用的条款；历史提交及文件名只能作为查找线索。不得把 cc-switch 或本项目根 MIT 当作全部商标许可。
2. **导入后修改或新增的 SVG**：保留修改前后版本；本轮用完整性验证过的 Lobe 1.73.0/1.98.0 比对，未匹配的继续留待确认，不靠视觉相似直接归属。
3. **PNG/JPG/WebP**：缺少来源元数据时，仅凭像素外观无法恢复作者或授权；向实际提交者索取素材取得记录。没有把未知图片转换格式或重画后宣布问题消失。
4. **OpenCode / new-api / Hermes**：对官方仓库固定 revision 的候选 logo/icon 文件进行了有界字节比对，未命中；不是穷尽历史或证明没有授权。逐项的 revision、候选数量、原因和建议已写入机器清单。
5. **拿不到可靠凭证**：建议后续用纯文字供应商名称或自行设计的非品牌通用图标替代，经功能与视觉审查再实施。本轮不删除/替换图标，不改变供应商识别逻辑。

逐项文件及内联符号的核查原因、处理建议见 [asset-provenance.json](asset-provenance.json) 的 `recheck` / `handling` 字段。库来源已确认的图标仍不等于单独商标授权已确认。
