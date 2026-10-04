package demo;
public enum C4 implements IOps {
    TIMES("*") { @Override public double apply(double x, double y){ return x*y; } },
    DIVIDE("/") { @Override public double apply(double x, double y){ return x/y; } };
    private final String t;
    C4(String t){ this.t=t; }
    public String getOp(){ return t; }
}
