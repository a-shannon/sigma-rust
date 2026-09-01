import XCTest

@testable import ErgoLib
@testable import ErgoLibC

final class RestNodeApiTests: XCTestCase {
    private enum SyntheticNodeError: Error, LocalizedError {
        case unavailable(URL)

        var errorDescription: String? {
            switch self {
            case .unavailable(let url):
                return "synthetic failure at \(url.absoluteString)"
            }
        }
    }

    func testFirstMainnetResultTriesUrlsInOrderUntilSuccess() async throws {
        let urls = [
            URL(string: "http://192.0.2.1:9053")!,
            URL(string: "http://192.0.2.2:9053")!,
        ]
        var attemptedUrls: [URL] = []

        let result: Int = try await firstMainnetResult(urls: urls) { url, _, _ in
            attemptedUrls.append(url)
            if url == urls[0] {
                throw SyntheticNodeError.unavailable(url)
            }
            return 42
        }

        XCTAssertEqual(result, 42)
        XCTAssertEqual(attemptedUrls, urls)
    }

    func testFirstMainnetResultThrowsExplicitFallbackError() async throws {
        let urls = [
            URL(string: "http://192.0.2.1:9053")!,
            URL(string: "http://192.0.2.2:9053")!,
        ]
        var attemptedUrls: [URL] = []

        do {
            let _: Int = try await firstMainnetResult(urls: urls) { url, _, _ in
                attemptedUrls.append(url)
                throw SyntheticNodeError.unavailable(url)
            }
            XCTFail("Expected every synthetic node attempt to fail")
        } catch let error as MainnetNodeFallbackError {
            XCTAssertEqual(attemptedUrls, urls)
            XCTAssertEqual(error.attemptedUrl, urls[1])
            XCTAssertEqual(
                error.underlyingErrorDescription,
                "synthetic failure at \(urls[1].absoluteString)"
            )
        } catch {
            XCTFail("Expected MainnetNodeFallbackError, got \(error)")
        }
    }

    func testExternalNodeAvailabilityClassifierAcceptsRequestAndTimeoutReasons() {
        let url = URL(string: "http://192.0.2.1:9053")!
        let requestReason = "ReqwestError(reqwest::Error { kind: Request, "
            + "source: ConnectError(\"connection refused\") })"
        let timeoutReason = "ReqwestError(reqwest::Error { kind: Request, source: TimedOut })"
        let timeoutError = MainnetNodeFallbackError(
            attemptedUrl: url,
            underlyingError: RestNodeApiError.misc(timeoutReason),
            underlyingErrorDescription: timeoutReason
        )

        XCTAssertEqual(
            externalNodeAvailabilityReason(for: RestNodeApiError.misc(requestReason)),
            requestReason
        )
        XCTAssertEqual(externalNodeAvailabilityReason(for: timeoutError), timeoutReason)
    }

    func testExternalNodeAvailabilityClassifierRejectsDecodeAndLocalErrors() {
        let url = URL(string: "http://192.0.2.1:9053")!
        let decodeReason = "ReqwestError(reqwest::Error { kind: Decode, "
            + "source: Error(\"request timed out while decoding\") })"
        let decodeError = MainnetNodeFallbackError(
            attemptedUrl: url,
            underlyingError: RestNodeApiError.misc(decodeReason),
            underlyingErrorDescription: decodeReason
        )
        let localError = MainnetNodeFallbackError(
            attemptedUrl: url,
            underlyingError: SyntheticNodeError.unavailable(url),
            underlyingErrorDescription: "synthetic failure"
        )

        XCTAssertNil(externalNodeAvailabilityReason(for: decodeError))
        XCTAssertNil(externalNodeAvailabilityReason(for: localError))
        XCTAssertNil(
            externalNodeAvailabilityReason(for: RestNodeApiError.misc("synthetic timeout"))
        )
    }

    func testGetNipopowProofByHeaderIdNonAsync() async throws {
        let expectation = self.expectation(description: "getNipopowByHeaderIdNonAsync")
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let callbackTask = Task {
            defer { expectation.fulfill() }
            return try await firstMainnetResult { _, restNodeApi, nodeConf in
                try await withCheckedThrowingContinuation { continuation in
                    do {
                        let _ = try restNodeApi.getNipopowProofByHeaderId(
                            nodeConf: nodeConf,
                            minChainLength: UInt32(3),
                            suffixLen: UInt32(2),
                            headerId: blockHeaders.get(index: UInt(0))!.getBlockId()
                        ) { (result: Result<NipopowProof, Error>) in
                            continuation.resume(with: result)
                        }
                    } catch {
                        continuation.resume(throwing: error)
                    }
                }
            }
        }

        // Allow two sequential 30-second node attempts plus callback delivery margin.
        let waiterResult = await XCTWaiter.fulfillment(of: [expectation], timeout: 75)
        switch waiterResult {
        case .completed:
            let proof = try await callbackTask.value
            XCTAssertNoThrow(try proof.toJSON()!)
        default:
            callbackTask.cancel()
            XCTFail("Callback did not complete before the waiter finished: \(waiterResult)")
            return
        }
    }

    func testGetNipopowProofByHeaderAbort() throws {
        let nodeConf = try NodeConf(withUrl: mainnetNodeUrls[0])
        let restNodeApi = try RestNodeApi()
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let handle = try restNodeApi.getNipopowProofByHeaderId(
            nodeConf: nodeConf,
            minChainLength: UInt32(3),
            suffixLen: UInt32(2),
            headerId: blockHeaders.get(index: UInt(0))!.getBlockId(),
            closure: { (res: Result<NipopowProof, Error>) -> Void in
                XCTFail("this should not be called")
            })
        handle.abort()
    }

    func testGetNipopowProofByHeaderIdAsync() async throws {
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let (proof, proofNew) = try await firstMainnetResult { _, restNodeApi, nodeConf in
            let proof = try await restNodeApi.getNipopowProofByHeaderIdAsync(
                nodeConf: nodeConf,
                minChainLength: UInt32(3),
                suffixLen: UInt32(2),
                headerId: blockHeaders.get(index: UInt(0))!.getBlockId()
            )

            // test re-use of the same Tokio runtime
            let proofNew = try await restNodeApi.getNipopowProofByHeaderIdAsync(
                nodeConf: nodeConf,
                minChainLength: UInt32(3),
                suffixLen: UInt32(2),
                headerId: blockHeaders.get(index: UInt(0))!.getBlockId()
            )
            return (proof, proofNew)
        }

        XCTAssertNoThrow(try proof.toJSON()!)
        XCTAssertNoThrow(try proofNew.toJSON()!)
    }

    func testPeerDiscoveryNonAsync() throws {
        let expectation = self.expectation(description: "peerDiscovery")
        let restNodeApi = try RestNodeApi()
        let _ = try restNodeApi.peerDiscovery(
            seeds: getSeeds(),
            maxParallelReqs: UInt16(30),
            timeoutSec: UInt32(3),
            closure: { (res: Result<CStringCollection, Error>) -> Void in
                switch res {
                case .success(let peers):
                    XCTAssert(peers.getLength() > 0)
                    break
                case .failure(let error):
                    XCTFail(error.localizedDescription)
                }
                expectation.fulfill()
            })
        waitForExpectations(timeout: 60, handler: nil)
    }

    func testPeerDiscoveryAsync() async throws {
        let restNodeApi = try RestNodeApi()
        let peers = try await restNodeApi.peerDiscoveryAsync(
            seeds: getSeeds(),
            maxParallelReqs: UInt16(30),
            timeoutSec: UInt32(3)
        )

        XCTAssert(!peers.isEmpty)

        // test of re-using of tokio runtime
        let peersNew = try await restNodeApi.peerDiscoveryAsync(
            seeds: getSeeds(),
            maxParallelReqs: UInt16(30),
            timeoutSec: UInt32(3)
        )
        XCTAssert(!peersNew.isEmpty)
    }

    func testSPVWorkflow() async throws {
        let headerId = try BlockId(
            withString: "d1366f762e46b7885496aaab0c42ec2950b0422d48aec3b91f45d4d0cdeb41e5")
        let txId = try TxId(
            withString: "258ddfc09b94b8313bca724de44a0d74010cab26de379be845713cc129546b78")
        // Get NiPoPow proofs from exactly 2 separate configured Ergo nodes.
        var proofs: [NipopowProof] = []
        var proofSourceUrls = Set<URL>()
        var externalNodeReasons: [String] = []
        for url in mainnetNodeUrls {
            if proofs.count >= 2 { break }
            if proofSourceUrls.contains(url) { continue }
            do {
                if let proof = try await getNipopowProof(url: url, headerId: headerId) {
                    proofs.append(proof)
                    proofSourceUrls.insert(url)
                } else {
                    externalNodeReasons.append(
                        "\(url.absoluteString): node does not expose the required NiPoPoW API"
                    )
                }
            } catch {
                guard let reason = externalNodeAvailabilityReason(for: error) else {
                    throw error
                }
                externalNodeReasons.append("\(url.absoluteString): \(reason)")
            }
        }
        guard proofs.count == 2 else {
            let diagnostics = externalNodeReasons.joined(separator: " | ")
            throw XCTSkip(
                "SPV workflow requires NiPoPoW proofs from two configured public nodes; "
                    + "received \(proofs.count). External-node diagnostics: \(diagnostics)"
            )
        }

        let genesisBlockId = try BlockId(
            withString: "b0244dfc267baca974a4caee06120321562784303a8a688976ae56170e4d175b")
        let verifier = NipopowVerifier(withGenesisBlockId: genesisBlockId)
        for proof in proofs {
            try verifier.process(newProof: proof)
        }
        let bestProof = verifier.bestProof()
        XCTAssertEqual(try bestProof.suffixHead().getHeader().getBlockId(), headerId)

        let (header, merkleProof) = try await firstMainnetResult {
            _, restNodeApi, nodeConf in
            let header = try await restNodeApi.getHeaderAsync(
                nodeConf: nodeConf,
                blockId: headerId
            )
            let merkleProof = try await restNodeApi.getBlocksHeaderIdProofForTxIdAsync(
                nodeConf: nodeConf,
                blockId: headerId,
                txId: txId
            )
            return (header, merkleProof)
        }
        XCTAssert(try merkleProof.valid(expected_root: header.getTransactionsRoot()))
    }
}
