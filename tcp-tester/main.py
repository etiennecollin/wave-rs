import socket
import sys


def main():
    if len(sys.argv) != 4:
        print("Usage: socket.py <address> <port> <message>")
        sys.exit(1)

    address = sys.argv[1]
    port = int(sys.argv[2])
    message = sys.argv[3]

    print(f"Sending message to {address}:{port}")

    try:
        # Create a TCP socket
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            # Connect to the specified address and port
            sock.connect((address, port))

            # Send the message
            sock.sendall(message.encode("utf-8"))

            print(f"Message sent to {address}:{port}")
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    main()
