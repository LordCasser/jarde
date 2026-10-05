public class MR {
    static class Node { Node next; String val; Node(String v, Node n){ val = v; next = n; } }
    static String legacyClose(java.lang.String arg0) throws java.io.IOException {
        java.io.StringReader local1;
        java.lang.String local3;
        local1 = null;
        try {
            local1 = new java.io.StringReader(arg0);
            int local2 = local1.read();
            local3 = "read:" + local2;
            return local3;
        } finally {
            if (local1 != null) {
                local1.close();
            }
        }
    }
    static String deepGuard(Node arg0) {
        if (arg0 != null) {
            if (arg0.next != null) {
                if (arg0.next.next != null) {
                    return arg0.next.next.val;
                }
            }
        }
        return "none";
    }
    public static void main(String[] a) throws Exception {
        Node n3 = new Node("deep", null); Node n2 = new Node("mid", n3); Node n1 = new Node("top", n2);
        System.out.println(""+legacyClose("AB")+"/"+deepGuard(n1)+"/"+deepGuard(n2)+"/"+deepGuard(null));
    }
}
