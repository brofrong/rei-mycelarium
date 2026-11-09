// Decompiles every gml_* function into <out>/objects|scripts/<name>.c and writes <out>/index.tsv.
// @category Mycelarium
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileOptions;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.decompiler.parallel.DecompileConfigurer;
import ghidra.app.decompiler.parallel.DecompilerCallback;
import ghidra.app.decompiler.parallel.ParallelDecompiler;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.Function;
import ghidra.util.task.TaskMonitor;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class ExportDecomp extends GhidraScript {
    @Override
    protected void run() throws Exception {
        String[] args = getScriptArgs();
        File out = new File(args.length > 0 ? args[0] : "re/decomp");
        new File(out, "objects").mkdirs();
        new File(out, "scripts").mkdirs();

        List<Function> funcs = new ArrayList<>();
        Map<String, Integer> seen = new HashMap<>();
        Map<Function, String> fileNames = new HashMap<>();
        for (Function f : currentProgram.getFunctionManager().getFunctions(true)) {
            String name = f.getName();
            if (!name.startsWith("gml_")) continue;
            int k = seen.merge(name, 1, Integer::sum);
            String file = k == 1 ? name : name + "_" + f.getEntryPoint();
            String dir = name.startsWith("gml_Object_") ? "objects" : "scripts";
            fileNames.put(f, dir + "/" + file + ".c");
            funcs.add(f);
        }

        DecompileConfigurer conf = d -> {
            DecompileOptions opts = new DecompileOptions();
            opts.grabFromProgram(currentProgram);
            d.setOptions(opts);
            d.toggleCCode(true);
            d.toggleSyntaxTree(true);
            d.setSimplificationStyle("decompile");
        };
        DecompilerCallback<String> cb = new DecompilerCallback<>(currentProgram, conf) {
            @Override
            public String process(DecompileResults res, TaskMonitor m) throws Exception {
                Function f = res.getFunction();
                String rel = fileNames.get(f);
                String status;
                if (res.decompileCompleted() && res.getDecompiledFunction() != null) {
                    String body = "// " + f.getName() + " @ 0x" + f.getEntryPoint() + "\n" + res.getDecompiledFunction().getC();
                    Files.writeString(new File(out, rel).toPath(), body, StandardCharsets.UTF_8);
                    status = "ok";
                } else {
                    String msg = res.getErrorMessage() == null ? "" : res.getErrorMessage();
                    status = msg.toLowerCase().contains("timeout") ? "timeout" : "error";
                }
                return f.getName() + "\t0x" + f.getEntryPoint() + "\t" + f.getBody().getNumAddresses() + "\t" + status + "\t" + rel;
            }
        };
        cb.setTimeout(120);
        List<String> rows;
        try {
            rows = new ArrayList<>(ParallelDecompiler.decompileFunctions(cb, funcs, monitor));
        } finally {
            cb.dispose();
        }
        Collections.sort(rows);
        StringBuilder sb = new StringBuilder("name\taddress\tsize\tstatus\tfile\n");
        int ok = 0;
        for (String r : rows) {
            sb.append(r).append('\n');
            if (r.split("\t")[3].equals("ok")) ok++;
        }
        Files.writeString(new File(out, "index.tsv").toPath(), sb.toString(), StandardCharsets.UTF_8);
        println("exported " + ok + "/" + rows.size() + " functions to " + out);
    }
}
