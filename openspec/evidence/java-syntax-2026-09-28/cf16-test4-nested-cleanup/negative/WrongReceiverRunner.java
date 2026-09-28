package jadx.tests.integration.trycatch;

import java.io.File;
import java.io.IOException;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.List;

public class WrongReceiverRunner {
    private static final class Probe extends OutputStream {
        private final String name;
        private final boolean failWrite;
        private final List<String> events;

        Probe(String name, boolean failWrite, List<String> events) {
            this.name = name;
            this.failWrite = failWrite;
            this.events = events;
        }

        @Override public void write(int value) throws IOException {
            events.add(name + ".write:" + value);
            if (failWrite) throw new IOException("body");
        }

        @Override public void close() {
            events.add(name + ".close");
        }
    }

    private static final class ProbeFile extends File {
        private static final long serialVersionUID = 1L;
        private final List<String> events;

        ProbeFile(List<String> events) {
            super("probe");
            this.events = events;
        }

        @Override public boolean delete() {
            events.add("file.delete");
            return true;
        }
    }

    public static void main(String[] args) {
        for (boolean failWrite : new boolean[] { false, true }) {
            List<String> events = new ArrayList<String>();
            String terminal = "none";
            try {
                WrongReceiver.run(new Probe("first", failWrite, events),
                    new Probe("other", false, events), new ProbeFile(events));
            } catch (IOException e) {
                terminal = "IOException:" + e.getMessage();
            }
            System.out.println((failWrite ? "body-io" : "normal") + "|" + events + "|" + terminal);
        }
    }
}
