public class M1 extends java.lang.Object {
    public M1() {
        super();
        return;
    }

    static int multi(java.lang.String arg0) {
        try {
            return java.lang.Integer.parseInt(arg0);
        } catch (java.lang.NumberFormatException | java.lang.NullPointerException local1) {
            return -1;
        } catch (java.lang.Exception local1) {
            return -2;
        }
    }

    static int labeled() {
        int local0;
        int local1;
        loop: for (local0 = 0; local0 < 3; local0 = local0 + 1) {
            for (local1 = 0; local1 < 3; local1 = local1 + 1) {
                if (local1 == 1) {
                    break;
                } else if (local0 == 2) {
                    break loop;
    }
            }
        }
        return 7;
    }

    static java.lang.String labeledSwitch(int arg0) {
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        loop: for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            switch (local2) {
                case 0:
                    local1.append('a');
                    break;
                case 1:
                    local1.append('b');
                    break loop;
                default:
                    local1.append('c');
                    break;
            }
        }
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println("" + multi("42") + "/" + multi("x") + "/" + multi((java.lang.String) null));
        java.lang.System.out.println(labeled());
        java.lang.System.out.println((java.lang.String) labeledSwitch(3));
        return;
    }
}
