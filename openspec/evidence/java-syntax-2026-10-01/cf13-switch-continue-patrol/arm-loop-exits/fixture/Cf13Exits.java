public class Cf13Exits {
    // v1: strict negative in javac-expressible form — every arm's exit is the latch and no
    // source continue exists anywhere: goto latch must stay a natural switch fall-out, never
    // a continue statement.
    public static int noJoinNoContinue(int n) {
        int total = 0;
        for (int i = 0; i < n; i++) {
            switch (i % 3) {
                case 0: total += 1; break;
                case 1: total += 2; break;
                default: total += 3;
            }
        }
        return total;
    }

    // v2: two arms each end in `if (…) continue;` and still complete at the shared join.
    public static int twoContinueArms(int n) {
        int total = 0;
        for (int i = 0; i < n; i++) {
            switch (i % 4) {
                case 0:
                    if (i > 4) continue;
                    total += 1;
                    break;
                case 1:
                    if (i > 5) continue;
                    total += 2;
                    break;
                case 2: total += 4; break;
                default: total += 3;
            }
            total += 10;
        }
        return total;
    }

    // v3: the whole default arm is one `continue;` statement.
    public static int defaultWholeContinue(int n) {
        int total = 0;
        for (int i = 0; i < n; i++) {
            switch (i % 3) {
                case 0: total += 1; break;
                case 1: total += 2; break;
                default: continue;
            }
            total += 10;
        }
        return total;
    }

    // v4: an arm leaves through the loop's own break channel (labeled break, the existing
    // LoopBreak route — regression only).
    public static int armBreaksLoop(int n) {
        int total = 0;
        loop: for (int i = 0; i < n; i++) {
            switch (i % 3) {
                case 0:
                    if (i > 3) break loop;
                    total += 1;
                    break;
                case 1: total += 2; break;
                default: total += 3;
            }
            total += 10;
        }
        return total;
    }

    // v5: continue in a while-form loop — the direct loop's continue target is the header test.
    public static int whileFormContinue(int n) {
        int total = 0;
        int i = 0;
        while (i < n) {
            switch (i % 3) {
                case 0: total += 1; break;
                case 1: total += 2; break;
                default:
                    if (i < 0) continue;
                    total += 3;
            }
            total += 10;
            i++;
        }
        return total;
    }

    // v6: negative — the arm edge reaches the OUTER loop's latch, not the direct loop's
    // latch/test (a labeled continue one level up): must keep the existing refusal.
    public static int outerLabeledContinue(int n) {
        int total = 0;
        outer: for (int i = 0; i < n; i++) {
            for (int j = 0; j < 3; j++) {
                switch (j % 3) {
                    case 0: total += 1; break;
                    default:
                        if (i > 1) continue outer;
                        total += 3;
                }
                total += 10;
            }
        }
        return total;
    }

    // v7: continue inside a string switch (regression: record the behavior as it is).
    public static int stringSwitchContinue(String[] args) {
        int total = 0;
        for (String s : args) {
            switch (s) {
                case "a": total += 1; break;
                case "b": total += 2; break;
                default:
                    if (s.isEmpty()) continue;
                    total += 3;
            }
            total += 10;
        }
        return total;
    }

    public static void main(String[] args) {
        System.out.println(noJoinNoContinue(7));
        System.out.println(twoContinueArms(7));
        System.out.println(defaultWholeContinue(7));
        System.out.println(armBreaksLoop(7));
        System.out.println(whileFormContinue(7));
        System.out.println(outerLabeledContinue(4));
        System.out.println(stringSwitchContinue(new String[] {"a", "b", "", "c", "a"}));
    }
}
