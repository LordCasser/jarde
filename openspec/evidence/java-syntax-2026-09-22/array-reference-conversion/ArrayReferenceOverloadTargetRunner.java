public class ArrayReferenceOverloadTargetRunner {
    private static void call(String name, Call call) {
        try {
            System.out.println(name + "=" + call.run());
        } catch (Throwable error) {
            System.out.println(name + "=error:" + error.getClass().getName());
        }
    }

    private interface Call {
        String run();
    }

    public static void main(String[] args) {
        final String[] empty = new String[0];
        final String[] sized = new String[3];
        final String[] absent = null;
        call("explicit-empty", new Call() {
            public String run() {
                return ArrayReferenceOverloadTarget.explicitObjectTarget(empty);
            }
        });
        call("explicit-sized", new Call() {
            public String run() {
                return ArrayReferenceOverloadTarget.explicitObjectTarget(sized);
            }
        });
        call("local-empty", new Call() {
            public String run() {
                return ArrayReferenceOverloadTarget.localObjectTarget(empty);
            }
        });
        call("local-sized", new Call() {
            public String run() {
                return ArrayReferenceOverloadTarget.localObjectTarget(sized);
            }
        });
        call("implicit-empty", new Call() {
            public String run() {
                return ArrayReferenceOverloadTarget.implicitStringTarget(empty);
            }
        });
        call("implicit-sized", new Call() {
            public String run() {
                return ArrayReferenceOverloadTarget.implicitStringTarget(sized);
            }
        });
        call("explicit-null", new Call() {
            public String run() {
                return ArrayReferenceOverloadTarget.explicitObjectTarget(absent);
            }
        });
        call("local-null", new Call() {
            public String run() {
                return ArrayReferenceOverloadTarget.localObjectTarget(absent);
            }
        });
        call("implicit-null", new Call() {
            public String run() {
                return ArrayReferenceOverloadTarget.implicitStringTarget(absent);
            }
        });
    }
}
