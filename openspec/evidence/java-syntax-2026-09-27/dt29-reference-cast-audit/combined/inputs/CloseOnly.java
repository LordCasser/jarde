package dt29;

import java.io.Closeable;
import java.io.IOException;

public class CloseOnly implements Closeable {
	@Override
	public void close() throws IOException {
	}
}
