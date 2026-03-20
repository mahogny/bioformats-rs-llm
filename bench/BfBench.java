/// Java Bio-Formats reader timing harness — called by bench/run.sh
///
/// Usage: java -cp <classpath> BfBench <path> <warmup_rounds> <measure_rounds>
/// Prints one line: average nanoseconds per iteration

import loci.formats.ImageReader;
import loci.common.DebugTools;

public class BfBench {
    public static void main(String[] args) throws Exception {
        DebugTools.setRootLevel("ERROR");
        String path       = args[0];
        int warmup        = Integer.parseInt(args[1]);
        int measure       = Integer.parseInt(args[2]);

        for (int w = 0; w < warmup; w++) {
            ImageReader r = new ImageReader();
            r.setId(path);
            for (int i = 0; i < r.getImageCount(); i++) r.openBytes(i);
            r.close();
        }

        long totalNs = 0;
        for (int m = 0; m < measure; m++) {
            ImageReader r = new ImageReader();
            long t0 = System.nanoTime();
            r.setId(path);
            for (int i = 0; i < r.getImageCount(); i++) r.openBytes(i);
            long t1 = System.nanoTime();
            r.close();
            totalNs += (t1 - t0);
        }

        System.out.println(totalNs / measure);
    }
}
