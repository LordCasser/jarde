import java.io.*;
public class MR {
    static String legacyClose(String path) throws IOException {                  // 旧式手动资源（pre-TWR）
        Reader r = null;
        try{
            r = new StringReader(path);
            int c = r.read();
            return "read:" + c;
        } finally {
            if(r != null){ r.close(); }
        }
    }
    static class Node { Node next; String val; Node(String v, Node n){ val = v; next = n; } }
    static String deepGuard(Node n){                                              // 深 null 守卫链
        if(n != null && n.next != null && n.next.next != null){ return n.next.next.val; }
        return "none";
    }
    public static void main(String[] a) throws Exception {
        Node n3 = new Node("deep", null); Node n2 = new Node("mid", n3); Node n1 = new Node("top", n2);
        System.out.println(""+legacyClose("AB")+"/"+deepGuard(n1)+"/"+deepGuard(n2)+"/"+deepGuard(null));
    }
}
