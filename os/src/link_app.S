.align 8
.data
.global _app_count
_app_count:
.quad 5
.quad app_0_name
.quad app_0_start
.quad app_1_name
.quad app_1_start
.quad app_2_name
.quad app_2_start
.quad app_3_name
.quad app_3_start
.quad app_4_name
.quad app_4_start
.quad app_4_end

.data
app_0_name:
  .string "00hello_world"
.align 8
app_0_start:
  .incbin "../user/target/x86_64/release/00hello_world.bin"
app_0_end:

.data
app_1_name:
  .string "01store_fault"
.align 8
app_1_start:
  .incbin "../user/target/x86_64/release/01store_fault.bin"
app_1_end:

.data
app_2_name:
  .string "02power"
.align 8
app_2_start:
  .incbin "../user/target/x86_64/release/02power.bin"
app_2_end:

.data
app_3_name:
  .string "03priv_inst"
.align 8
app_3_start:
  .incbin "../user/target/x86_64/release/03priv_inst.bin"
app_3_end:

.data
app_4_name:
  .string "04priv_regs"
.align 8
app_4_start:
  .incbin "../user/target/x86_64/release/04priv_regs.bin"
app_4_end:
