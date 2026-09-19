/** A module that both uses and provides one service. */
module p4.sample {
    uses java.util.spi.ToolProvider;
    provides java.util.spi.ToolProvider with p4.sample.Provider;
}
