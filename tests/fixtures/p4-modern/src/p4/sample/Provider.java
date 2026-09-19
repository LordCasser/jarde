package p4.sample;

/** The provider the sample module declares with `provides`. */
public class Provider implements java.util.spi.ToolProvider {
    @Override
    public String name() {
        return "p4";
    }

    @Override
    public int run(java.io.PrintWriter out, java.io.PrintWriter err, String... args) {
        return 0;
    }
}
