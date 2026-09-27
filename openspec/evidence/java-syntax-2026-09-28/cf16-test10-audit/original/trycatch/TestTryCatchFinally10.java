package trycatch;

import android.content.Context;
import java.io.IOException;
import java.io.InputStream;
import java.util.Scanner;
import log.DebugLogger;

public class TestTryCatchFinally10 {
	static final DebugLogger l = new DebugLogger();

	public static String test(Context context, int i) {
		CommonContracts.requireNonNull(context);
		InputStream inputStream = null;
		try {
			inputStream = context.getResources().openRawResource(i);
			Scanner useDelimiter = new Scanner(inputStream).useDelimiter("\\A");
			return useDelimiter.hasNext() ? useDelimiter.next() : "";
		} finally {
			if (inputStream != null) {
				try {
					inputStream.close();
				} catch (IOException e) {
					l.logException(DebugLogger.LogLevel.ERROR, e);
				}
			}
		}
	}
}
