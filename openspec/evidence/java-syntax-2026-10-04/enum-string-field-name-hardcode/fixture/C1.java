package demo;
public enum C1 implements IOps {
    TIMES("*") { @Override public double apply(double x, double y){ return x*y; } },
    DIVIDE("/") { @Override public double apply(double x, double y){ return x/y; } };
    private final String op;
    C1(String op){ this.op=op; }
    public String getOp(){ return op; }
}
