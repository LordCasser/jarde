package flow;

public class HeaderShadow<flow> extends GenericParent<flow> {
    public flow value;
    public java.util.List<flow> nested;

    public flow echo(flow value) { return value; }

    public HeaderShadow() {
        super();
    }
}
