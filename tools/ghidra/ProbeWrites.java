// Prints instructions around write references to given addresses (debug helper).
// @category Mycelarium
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Instruction;
import ghidra.program.model.symbol.Reference;

public class ProbeWrites extends GhidraScript {
    @Override
    protected void run() throws Exception {
        for (String a : getScriptArgs()) {
            Address addr = toAddr(Long.parseLong(a, 16));
            println("== " + a);
            for (Reference r : getReferencesTo(addr)) {
                if (getFunctionContaining(r.getFromAddress()) != null && getFunctionContaining(r.getFromAddress()).getName().startsWith("gml_")) continue;
                Instruction ins = getInstructionAt(r.getFromAddress());
                Instruction p = ins;
                for (int i = 0; i < 6 && p.getPrevious() != null; i++) p = p.getPrevious();
                StringBuilder sb = new StringBuilder("  in " + getFunctionContaining(r.getFromAddress()) + "\n");
                for (int i = 0; i < 9 && p != null; i++, p = p.getNext()) sb.append("    ").append(p.getAddress()).append("  ").append(p).append("\n");
                println(sb.toString());
                break;
            }
        }
    }
}
