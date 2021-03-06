"""
The `cretonne.entities` module predefines all the Cretonne entity reference
operand types. There are corresponding definitions in the `cretonne.entities`
Rust module.
"""
from __future__ import absolute_import
from cdsl.operands import EntityRefKind


#: A reference to an extended basic block in the same function.
#: This is primarliy used in control flow instructions.
ebb = EntityRefKind(
        'ebb', 'An extended basic block in the same function.',
        default_member='destination')

#: A reference to a stack slot declared in the function preamble.
stack_slot = EntityRefKind('stack_slot', 'A stack slot.')

#: A reference to a function sugnature declared in the function preamble.
#: Tbis is used to provide the call signature in an indirect call instruction.
sig_ref = EntityRefKind('sig_ref', 'A function signature.')

#: A reference to an external function declared in the function preamble.
#: This is used to provide the callee and signature in a call instruction.
func_ref = EntityRefKind('func_ref', 'An external function.')

#: A reference to a jump table declared in the function preamble.
jump_table = EntityRefKind(
        'jump_table', 'A jump table.', default_member='table')
