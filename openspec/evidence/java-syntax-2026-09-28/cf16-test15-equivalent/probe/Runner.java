package trycatch;

import android.os.IBinder;
import android.os.Parcel;
import android.os.RemoteException;

public final class Runner {
	public static void main(String[] args) throws Exception {
		String mode = args[0];
		Parcel.EVENTS.setLength(0);
		Parcel.failRead = "read".equals(mode);
		Parcel.failRecycleOnce = "cleanup".equals(mode);
		Parcel input = new Parcel("in");
		IBinder binder = (code, in, out, flags) -> {
			Parcel.EVENTS.append("transact;");
			if ("runtime".equals(mode)) {
				throw new IllegalStateException("transact");
			}
			if ("remote".equals(mode)) {
				throw new RemoteException("transact");
			}
			return true;
		};
		try {
			Parcel result = new TestTryCatchFinally15(binder).test(7, input);
			Parcel.EVENTS.append("return:").append(result.name()).append(';');
		} catch (Throwable throwable) {
			Parcel.EVENTS.append("throw:")
					.append(throwable.getClass().getSimpleName())
					.append(':').append(throwable.getMessage()).append(';');
		}
		System.out.println(Parcel.EVENTS);
	}
}
