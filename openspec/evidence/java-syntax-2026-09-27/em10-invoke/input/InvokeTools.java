package em10;

class InvokeTools {
	static long combine(int first, long second, double third, int fourth) {
		return first + second + (long) third + fourth;
	}

	static String caught(String message) {
		return "caught:" + message;
	}
}
