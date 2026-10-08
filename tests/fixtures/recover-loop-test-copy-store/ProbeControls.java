import java.io.*;
public class ProbeControls {
    static int parameterTarget(int c, String path) throws IOException {      // the target is a parameter
        FileReader fr = new FileReader(path);
        int n = 0;
        while ((c = fr.read()) != -1) { n = n + c; }
        fr.close();
        return n + c;
    }
    static int guardIfFirst(String path) throws IOException {                // an if-position dance in the guard body
        FileReader fr = new FileReader(path);
        int total = 0;
        try {
            if ((total = fr.read()) > 0) { total = total + 1; }
            int c;
            while ((c = fr.read()) != -1) { total = total + c; }
        } finally { fr.close(); }
        return total;
    }
    public static void main(String[] a) throws Exception {
        System.out.println(parameterTarget(0, "data.txt") + "/" + guardIfFirst("data.txt"));
    }
}
