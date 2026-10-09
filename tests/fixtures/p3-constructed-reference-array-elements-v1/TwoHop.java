public class TwoHop extends Mid implements LocalInterface {
    public TwoHop(String tag) {
        super(tag);
        Main.event("ctor:TwoHop", tag);
    }
}
