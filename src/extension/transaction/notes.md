# Gordian Sealed Transaction Protocol (GSTP): Design Overview

Wolf McNally, Blockchain Commons

## Overview

This document describes the high-level goals of Gordian Sealed Transaction Protocol (GSTP), s secure message-passing system that enables stateful and efficient communication between peers. The system utilizes encrypted messages, signatures, message-encapsulated state, and idempotent actions to provide a robust and scalable communication model. It also includes mechanisms to prevent the misuse, malicious manipulation, or replay of responses.

## Terminology

- **Message:** A unit of communication between peers.
- **Sender:** The peer sending a message.
- **Receiver:** The peer receiving a message.
- **Request:** A message sent by a peer to request an action or information from another peer (or peers).
- **Response:** A message sent by a peer in response to a request, containing a result or an error.
- **Requester:** The peer sending a request.
- **Responder:** The peer responding to a request.
- **Continuation:** A unit of private, self-encrypted state information included in a message, allowing the sender to resume a computation.
- **Idempotent:** An action that has the same effect when performed multiple times as when performed once.
- **TOFU:** Trust-On-First-Use, a security mechanism that relies on the first exchange of public keys to establish trust.
- **Horizontal Scalability:** The ability of a system to handle increased load by adding more resources.

## Key Components

- **Asynchronous Message-Passing:** The peers communicate asynchronously, allowing for non-blocking and concurrent message exchange.

- **Request-Response Pair IDs:** Each request-response pair has a unique ID, enabling the receiver to match responses to the corresponding requests, and if necessary route responses back to a requesting process.

- **Encrypted Messages:** Each peer possesses a key pair and encrypts messages using the recipient's public key, ensuring confidentiality and security. In the case of multicast messages, the sender may encrypt the message to multiple recipients. In the case of public broadcast messages, the sender may forego encryption.

- **Self-Signed Requests:** The requester includes their public key in each request, and signs the message with their private key. The responder can verify the signature and the sender's public key, ensuring the integrity and authenticity of the message. Including the requester's public key in the message also enables Trust-On-First-Use (TOFU) authentication. Responses are also signed by the responder.

- **Message-Encapsulated State:** Senders may include a self-encrypted "continuation" in messages, containing private state information needed to resume a computation. The receiver cannot decrypt the continuation and must return it unaltered so the requester can resume the computation.

- **Inclusion of Request ID in Continuation** To prevent replays of continuations, a requester includes the request ID in their continuation. The requester can verify that the continuation returned in the response matches the response ID and therefore corresponds to the original request, without having to keep track of the original request ID locally. Continuations provided by responders do not include the request ID, as they may be returned in a future request.

- **Continuation Timeout:** Continuations may include a date stamp representing a timeout. When receiving a continuation back, the receiver checks it against the current time and rejects the entire message if the continuation has expired.

- **Idempotent Actions:** It is strongly encouraged that individual actions within the system are designed to be idempotent, meaning that performing the same action multiple times has the same effect as performing it once. This ensures consistency and fault tolerance. For example, a message to turn a light on is idempotent because turning the light on multiple times has the same effect as turning it on once. A message to toggle a light's state is not idempotent because toggling the light multiple times has a different effect than toggling it once.

## Benefits

- **Reduced State Management:** Continuations allow peers to offload state information into the passed messages, minimizing (ideally eliminating) the amount of local state needed for ongoing tasks.

- **Fault Tolerance:** Idempotency of actions ensures that the system remains consistent even if actions are repeated due to lost messages.

- **Scalability:** Responders can process requests independently and return continuations, enabling horizontal scalability and load distribution.

- **Security:** Self-encrypted continuations maintain the privacy and integrity of the sender's state information. The inclusion of request IDs and timeouts in continuations helps prevent misuse or malicious manipulation of continuations, e.g., replay attacks.

- **Efficient Data Handling:** Continuations in responses enable pagination, streaming, and incremental processing of large result sets, optimizing resource utilization and performance.

## Usage Scenarios

- **Stateless Multi-Step Interactions:** Using message-encapsulated state through continuations, the system supports multi-step operations or workflows that require multiple request-response cycles.

- **Large Data Transfers:** Continuations in responses allow for efficient transfer and processing of large data sets, enabling pagination and streaming.

- **Long-running Operations:** The responder can send partial results along with continuations, allowing the requester to process results incrementally.

## Conclusion

GSTP provides a secure, scalable, and efficient communication model for interactions between distributed peers. The inclusion of request-response pair IDs, continuation timeouts, and the idempotency of actions significantly mitigates the risks of misuse, malicious manipulation, and replay attacks. The system ensures the privacy, integrity, and reliability of the exchanged information while supporting scenarios involving large data transfers, long-running operations, and multi-step workflows. The combination of security measures and efficient data handling makes this system well-suited for secure and reliable communication between peers in various contexts.
