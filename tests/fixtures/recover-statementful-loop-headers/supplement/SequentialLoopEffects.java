public final class SequentialLoopEffects {
    public int c;
    public int z;
    public int phase;
    public int evaluations;
    public int phaseOneCalls;
    public int phaseTwoCalls;
    public int lastInputC;
    public int lastInputA;
    public int lastValue;
    public int throwAt;
    public int overrideAt;
    public int overrideValue;
    public long phaseTrace;
    public final RuntimeException marker;

    public SequentialLoopEffects() {
        this.marker = new RuntimeException("scheduled");
    }

    private int next(int current, int a) {
        evaluations++;
        if (phase == 1) {
            phaseOneCalls++;
        } else if (phase == 2) {
            phaseTwoCalls++;
        }
        phaseTrace = (phaseTrace << 2) | phase;
        lastInputC = current;
        lastInputA = a;
        int value = current + a;
        if (evaluations == overrideAt) {
            value = overrideValue;
        }
        lastValue = value;
        if (evaluations == throwAt) {
            throw marker;
        }
        return value;
    }

    public int run(int a, int initialC) {
        c = initialC;
        phase = 1;
        while (true) {
            z = next(c, a);
            if (z >= 7) {
                break;
            }
            c = z;
        }
        phase = 2;
        while ((z = next(c, a)) >= 7) {
            c = z;
        }
        return c;
    }
}
