public class SelfArgument {
    public static int pick(Object value) { return 1; }
    public static int pick(SelfArgument value) { return 2; }
    public int run() { return pick((Object) this); }
}
