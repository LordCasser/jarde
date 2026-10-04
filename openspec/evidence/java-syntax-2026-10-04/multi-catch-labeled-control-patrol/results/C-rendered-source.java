public class C extends java.lang.Object {
    public C() {
        super();
        return;
    }

    static java.lang.String trail() {
        java.lang.StringBuilder local0;
        int local1;
        local0 = new java.lang.StringBuilder();
        int local2;
        loop: for (local1 = 0; local1 < 2; local1 = local1 + 1) {
            local2 = 0;
            while (local2 < 2) {
                if (local2 == 1) {
                    continue loop;
                } else {
                    local0.append("in" + local1 + local2 + ";");
                    local2 = local2 + 1;
                }
            }
            local0.append("after" + local1 + ";");
        }
        return local0.toString();
    }

    static java.lang.String trailBreak() {
        java.lang.StringBuilder local0;
        int local1;
        local0 = new java.lang.StringBuilder();
        local1 = 0;
        int local2;
        loop: while (local1 < 2) {
            local2 = 0;
            while (local2 < 2) {
                if (local2 == 1) {
                    break loop;
                } else {
                    local0.append("in" + local1 + local2 + ";");
                    local2 = local2 + 1;
                }
            }
            local0.append("after" + local1 + ";");
            local1 = local1 + 1;
        }
        return local0.toString();
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println("trail=" + trail());
        java.lang.System.out.println("trailBreak=" + trailBreak());
        return;
    }
}
