// Names YYC functions (gml_*), runtime builtins (F_<name>) and user variable id slots (vid_<name>).
// @category Mycelarium
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.Instruction;
import ghidra.program.model.mem.Memory;
import ghidra.program.model.mem.MemoryBlock;
import ghidra.program.model.symbol.Reference;
import ghidra.program.model.symbol.SourceType;

import java.util.ArrayList;
import java.util.List;

public class NameYYC extends GhidraScript {
    private Memory mem;

    @Override
    protected void run() throws Exception {
        mem = currentProgram.getMemory();
        int gml = nameGmlFunctions();
        int builtins = nameBuiltins();
        int vars = nameVariableSlots();
        nameRuntimeHelpers();
        writeBuiltinList();
        println("named gml=" + gml + " builtins=" + builtins + " vars=" + vars);
    }

    // YYC runtime helpers identified by hand (see re/notes/yyc-runtime.md).
    static final String[][] HELPERS = {
        {"1400243e0", "FREE_RValue"},
        {"140024510", "COPY_RValue"},
        {"140024340", "COPY_RValue_Post"},
        {"1406db110", "YYGML_CallBuiltin"},
        {"14070edc0", "YYCompareVal"},
        {"1406dacd0", "YYError"},
    };
    static final String[][] GLOBALS = {
        {"141020570", "g_pGlobal"},
        {"141379c38", "g_pCurrentStackTrace"},
    };

    void nameRuntimeHelpers() throws Exception {
        for (String[] h : HELPERS) {
            Function f = getFunctionAt(toAddr(Long.parseLong(h[0], 16)));
            if (f != null) rename(f, h[1]);
        }
        for (String[] g : GLOBALS) {
            createLabel(toAddr(Long.parseLong(g[0], 16)), g[1], true, SourceType.USER_DEFINED);
        }
    }

    void writeBuiltinList() throws Exception {
        String[] args = getScriptArgs();
        java.io.File out = new java.io.File(args.length > 0 ? args[0] : "re", "runtime_builtins.txt");
        out.getParentFile().mkdirs();
        java.util.TreeSet<String> names = new java.util.TreeSet<>();
        for (Function f : currentProgram.getFunctionManager().getFunctions(true)) {
            if (f.getName().startsWith("F_")) names.add(f.getName().substring(2));
        }
        java.nio.file.Files.writeString(out.toPath(), String.join("\n", names) + "\n");
    }

    static String sanitize(String s) {
        return s.replace("@", "__");
    }

    String cString(Address a) {
        StringBuilder sb = new StringBuilder();
        try {
            for (int i = 0; i < 512; i++) {
                byte b = mem.getByte(a.add(i));
                if (b == 0) return sb.toString();
                if (b < 0x20 || b > 0x7e) return null;
                sb.append((char) b);
            }
        } catch (Exception e) {
            return null;
        }
        return null;
    }

    List<Address> findStrings(String prefix) throws Exception {
        List<Address> out = new ArrayList<>();
        byte[] needle = ("\0" + prefix).getBytes("ASCII");
        for (MemoryBlock blk : mem.getBlocks()) {
            if (!blk.isInitialized() || blk.isExecute()) continue;
            Address cur = blk.getStart();
            while (cur != null && cur.compareTo(blk.getEnd()) < 0) {
                Address hit = mem.findBytes(cur, blk.getEnd(), needle, null, true, monitor);
                if (hit == null) break;
                out.add(hit.add(1));
                cur = hit.add(1);
            }
        }
        return out;
    }

    void rename(Function f, String name) throws Exception {
        if (f.getName().equals(name)) return;
        f.setName(name, SourceType.USER_DEFINED);
    }

    int nameGmlFunctions() throws Exception {
        int n = 0;
        for (Address s : findStrings("gml_")) {
            String name = cString(s);
            if (name == null || name.contains("%")) continue;
            for (Reference r : getReferencesTo(s)) {
                Function f = getFunctionContaining(r.getFromAddress());
                if (f == null) continue;
                long delta = r.getFromAddress().subtract(f.getEntryPoint());
                if (delta >= 0 && delta < 0x100 && !f.getName().startsWith("gml_")) {
                    rename(f, sanitize(name));
                    n++;
                }
            }
        }
        return n;
    }

    Address leaTarget(Instruction ins, String reg) {
        if (!ins.getMnemonicString().equals("LEA")) return null;
        Object[] dst = ins.getOpObjects(0);
        if (dst.length != 1 || !dst[0].toString().equalsIgnoreCase(reg)) return null;
        Reference ref = ins.getPrimaryReference(1);
        if (ref != null) return ref.getToAddress();
        for (Object o : ins.getOpObjects(1)) {
            if (o instanceof Address) return (Address) o;
        }
        return null;
    }

    int nameBuiltins() throws Exception {
        Address functionAdd = null;
        for (Address s : findStrings("draw_sprite_ext")) {
            if (!"draw_sprite_ext".equals(cString(s))) continue;
            for (Reference r : getReferencesTo(s)) {
                Instruction ins = getInstructionAt(r.getFromAddress());
                for (int i = 0; ins != null && i < 8; i++, ins = ins.getNext()) {
                    if (ins.getFlowType().isCall()) {
                        Address[] flows = ins.getFlows();
                        if (flows.length == 1) functionAdd = flows[0];
                        break;
                    }
                }
                if (functionAdd != null) break;
            }
            if (functionAdd != null) break;
        }
        if (functionAdd == null) {
            println("Function_Add not found");
            return 0;
        }
        Function fa = getFunctionAt(functionAdd);
        if (fa != null) rename(fa, "Function_Add");
        int n = 0;
        for (Reference call : getReferencesTo(functionAdd)) {
            if (!call.getReferenceType().isCall()) continue;
            Instruction ins = getInstructionAt(call.getFromAddress());
            Address nameAddr = null, funcAddr = null;
            Instruction p = ins == null ? null : ins.getPrevious();
            for (int i = 0; p != null && i < 8; i++, p = p.getPrevious()) {
                if (p.getFlowType().isCall()) break;
                if (nameAddr == null) nameAddr = leaTarget(p, "RCX");
                if (funcAddr == null) funcAddr = leaTarget(p, "RDX");
            }
            if (nameAddr == null || funcAddr == null) continue;
            String name = cString(nameAddr);
            if (name == null || name.isEmpty()) continue;
            Function f = getFunctionAt(funcAddr);
            if (f == null) f = createFunction(funcAddr, null);
            if (f == null) continue;
            if (!f.getName().startsWith("F_")) {
                rename(f, "F_" + name);
                n++;
            }
        }
        return n;
    }

    boolean isVarEntry(Address e) {
        try {
            long ptr = mem.getLong(e);
            long id = mem.getLong(e.add(8));
            if (id != 0xffffffffL && (id < 0 || id >= 0x1000000L)) return false;
            String s = cString(toAddr(ptr));
            return s != null && !s.isEmpty();
        } catch (Exception ex) {
            return false;
        }
    }

    int nameVariableSlots() throws Exception {
        Address seed = null;
        for (Address s : findStrings("chunk_w")) {
            if (!"chunk_w".equals(cString(s))) continue;
            byte[] ptr = new byte[8];
            long v = s.getOffset();
            for (int i = 0; i < 8; i++) ptr[i] = (byte) (v >>> (8 * i));
            for (MemoryBlock blk : mem.getBlocks()) {
                if (!blk.isInitialized() || blk.isExecute()) continue;
                Address hit = mem.findBytes(blk.getStart(), blk.getEnd(), ptr, null, true, monitor);
                if (hit != null && isVarEntry(hit)) {
                    seed = hit;
                    break;
                }
            }
            if (seed != null) break;
        }
        if (seed == null) {
            println("variable table not found");
            return 0;
        }
        Address start = seed;
        while (isVarEntry(start.subtract(16))) start = start.subtract(16);
        int n = 0;
        for (Address e = start; isVarEntry(e); e = e.add(16)) {
            String name = cString(toAddr(mem.getLong(e)));
            createLabel(e.add(8), "vid_" + sanitize(name), true, SourceType.USER_DEFINED);
            n++;
        }
        return n;
    }
}
