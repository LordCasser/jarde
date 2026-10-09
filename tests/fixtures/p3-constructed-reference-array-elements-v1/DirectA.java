public class DirectA extends Base implements LocalInterface {
    public DirectA(String tag) {
        super(tag);
        Main.event("ctor:DirectA", tag);
    }
}
