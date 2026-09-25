import java.util.List;
public class AmbiguousInnerBoundary {
    public static class Left { public static class Entry {} }
    public static class Right { public static class Entry {} }
    public static List<Left.Entry> left(List<Left.Entry> values) { return values; }
    public static List<Right.Entry> right(List<Right.Entry> values) { return values; }
}
