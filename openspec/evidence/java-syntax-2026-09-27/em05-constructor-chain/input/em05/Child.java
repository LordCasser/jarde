package em05;
public class Child extends Base {
    public Child() { super(); }
    public Child(String value) { super(value); }
    public Child(int value) { this(java.lang.Integer.toString(value)); }
}
