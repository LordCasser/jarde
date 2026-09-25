public class FinalInit {
    public static final int CONSTANT = 5;
    public static final int COMPUTED = compute();
    public static final Object OBJECT = new Object();
    public final int instance;
    public FinalInit(int value) { instance = value; }
    public static int compute() { return 7; }
}
