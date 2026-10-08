import java.io.*;
public class Probe {
    static String readAll(String path) throws IOException {                  // the same-form loop, no guard
        FileReader fr = new FileReader(path);
        StringBuilder sb = new StringBuilder();
        int c;
        while ((c = fr.read()) != -1) { sb.append((char) c); }
        fr.close();
        return sb.toString();
    }
    static int guardPlain(String path) throws IOException {                  // the control: a guard body's if walks
        FileReader fr = new FileReader(path);
        int total = 0;
        try {
            total = fr.read();
            if (total > 0) { total = total + 1; }
            int c;
            while ((c = fr.read()) != -1) { total = total + c; }
        } finally { fr.close(); }
        return total;
    }
    public static void main(String[] a) throws Exception {
        System.out.println(readAll("data.txt").replace("\n", "|") + "/" + guardPlain("data.txt"));
    }
}
