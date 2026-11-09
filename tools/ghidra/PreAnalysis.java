// Turns off slow analyzers that the YYC symbolization does not need.
// @category Mycelarium
import ghidra.app.script.GhidraScript;

public class PreAnalysis extends GhidraScript {
    @Override
    protected void run() throws Exception {
        setAnalysisOption(currentProgram, "Decompiler Parameter ID", "false");
        setAnalysisOption(currentProgram, "Aggressive Instruction Finder", "false");
    }
}
