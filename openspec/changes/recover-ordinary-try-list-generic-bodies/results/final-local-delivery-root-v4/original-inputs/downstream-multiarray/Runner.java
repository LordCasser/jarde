public final class Runner {
    public static void main(String[] args) {
        CarryUnprovedConsumerProbe.trace = "";
        CarryUnprovedConsumerProbe.effects = 0;
        CarryUnprovedConsumerProbe.candidate(false);
        System.out.println("false|" + CarryUnprovedConsumerProbe.effects + "|" + CarryUnprovedConsumerProbe.trace);

        CarryUnprovedConsumerProbe.trace = "";
        CarryUnprovedConsumerProbe.effects = 0;
        CarryUnprovedConsumerProbe.candidate(true);
        System.out.println("true|" + CarryUnprovedConsumerProbe.effects + "|" + CarryUnprovedConsumerProbe.trace);
    }
}
