# judge MUST 审查记录：verdict-260919-03

- 审查者：judge subagent（34e59edb，2026-09-19）
- 判定：**PASS-with-conditions**（三源核对全部对上，数值零偏差；建议 S1 补后可授 candidate）
- 三源：①verdict 快照 ②git 链 a376b4c→ebcb393→0615298（窗位实证 jni_bridge.rs:575-614、
  build.gradle:111-112 映射、Mixin sysprop 消费）③四臂 result.json 独立重算
  （nativeMs P50 3.410/3.370/4.224/4.012；ratios median 0.9803/0.9813/0.9814/0.9818；LP P50 零偏差）
- S1（已应用）：verdict 补 packed_coverage=1−inline 占比换算声明（M1 指标补数方向防误读）。
- S2（登记处置）：sha8=7519ddb8 构建补记实际与实现同 commit（0615298，16:04），非独立时点——
  时序锚本质成立（判据阈值 commit a376b4c/ebcb393 先于实现与采集；cmd-output 不入库故采集在
  其后）；判据已冻结不回改，本条即为表述精确化登记。
- S3：onpk 参考臂 fill_us 未并列（完整性提示，无需动作）。
- judge 意见：观测事实（B-主 未满足 1.177 / B-C3 满足 0.155 / B-C4 灰区 0.988 / 等价门全绿 /
  机制归因窗位实证）证据链完整；**建议可授 candidate**；C-3 回退缺省关、C-4 保留属工程决策留用户。
- 无 halt、无 retry 超限、无噪声卡冲突信号。
