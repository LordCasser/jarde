package controls;

/** Minimal independent control for the loop-condition and char-to-int assignment. */
public final class LoopConditionInvokeControl {
    private static final char STOP_CHAR = 0;

    private final char[] input;
    private int pos;
    private int calls;
    private int events;

    public LoopConditionInvokeControl(char[] input, int startingPos) {
        this.input = input;
        this.pos = startingPos;
    }

    public boolean test(char lastChar) {
        int startPos = pos;
        int ch;
        while ((ch = next()) != STOP_CHAR) {
            if (ch == lastChar) {
                return true;
            }
        }
        pos = startPos;
        return false;
    }

    private char next() {
        calls++;
        int index = pos++;
        events = events * 10 + index + 1;
        return input[index];
    }

    public int getPos() {
        return pos;
    }

    public int getCalls() {
        return calls;
    }

    public int getEvents() {
        return events;
    }
}
