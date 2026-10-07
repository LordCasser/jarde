import java.io.*;
public class IO {
    static int countLines(String path) throws IOException {                       // 三层包装 + readLine 循环
        BufferedReader r = new BufferedReader(new InputStreamReader(new FileInputStream(path), "UTF-8"));
        try {
            int n = 0;
            String line;
            while((line = r.readLine()) != null){ n++; }
            return n;
        } finally {
            r.close();
        }
    }
    static String readAll(String path) throws IOException {                        // StringBuilder 累积 + 手动 char 循环
        FileReader fr = new FileReader(path);
        StringBuilder sb = new StringBuilder();
        try {
            int c;
            while((c = fr.read()) != -1){ sb.append((char) c); }
        } finally { fr.close(); }
        return sb.toString();
    }
    public static void main(String[] a) throws Exception {
        System.out.println(""+countLines("data.txt")+"/"+readAll("data.txt").replace("\n","|"));
    }
}
