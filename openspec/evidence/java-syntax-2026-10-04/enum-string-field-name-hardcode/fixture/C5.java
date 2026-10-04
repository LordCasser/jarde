package demo;
public enum C5 implements IOps {
    TIMES("*") { @Override public double apply(double x, double y){ return x*y; } },
    DIVIDE("/") { @Override public double apply(double x, double y){ return x/y; } };
    private final String value;
    C5(String value){ this.value=value; }
    public String getValue(){ return value; }
}
