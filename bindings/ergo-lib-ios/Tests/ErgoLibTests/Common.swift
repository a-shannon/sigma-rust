import Foundation

@testable import ErgoLib
@testable import ErgoLibC

// The following two functions copied from: https://stackoverflow.com/a/49890939

func toPairsOfChars(pairs: [String], string: String) -> [String] {
    if string.count == 0 {
        return pairs
    }
    var pairsMod = pairs
    pairsMod.append(String(string.prefix(2)))
    return toPairsOfChars(pairs: pairsMod, string: String(string.dropFirst(2)))
}

func base16StringToBytes(_ string: String) -> [UInt8]? {
    // omit error checking: remove '0x', make sure even, valid chars
    let pairs = toPairsOfChars(pairs: [], string: string)
    return pairs.map { UInt8($0, radix: 16)! }
}

func getSeeds() -> [URL] {
    [
        "http://213.239.193.208:9030",
        "http://159.65.11.55:9030",
        "http://165.227.26.175:9030",
        "http://159.89.116.15:9030",
        "http://136.244.110.145:9030",
        "http://94.130.108.35:9030",
        "http://51.75.147.1:9020",
        "http://221.165.214.185:9030",
        "http://51.81.185.231:9031",
        "http://217.182.197.196:9030",
        "http://62.171.190.193:9030",
        "http://173.212.220.9:9030",
        "http://176.9.65.58:9130",
        "http://213.152.106.56:9030",
    ].map { URL(string: $0)! }
}

func getNipopowProof(url: URL, headerId: BlockId) async throws -> NipopowProof? {
    let nodeConf = try NodeConf(withUrl: url)
    let restNodeApi = try RestNodeApi()
    let nodeInfo = try await restNodeApi.getInfoAsync(nodeConf: nodeConf)
    if nodeInfo.isAtLeastVersion40100() {
        let proof = try await restNodeApi.getNipopowProofByHeaderIdAsync(
            nodeConf: nodeConf, minChainLength: UInt32(7), suffixLen: UInt32(6), headerId: headerId)
        return proof
    } else {
        return nil
    }
}

// Known-live mainnet nodes (REST API on :9053) used as fallbacks for network-dependent tests.
let mainnetNodeUrls = [
    "http://213.239.193.208:9053",
    "http://159.65.11.55:9053",
].map { URL(string: $0)! }

/// Returns a NodeConf for the first mainnet node that answers a getInfo request.
/// Throws the last error if no node is reachable.
func reachableNodeConf() async throws -> NodeConf {
    var lastError: Error?
    for url in mainnetNodeUrls {
        do {
            let nodeConf = try NodeConf(withUrl: url)
            let restNodeApi = try RestNodeApi()
            let _ = try await restNodeApi.getInfoAsync(nodeConf: nodeConf)
            return nodeConf
        } catch {
            lastError = error
        }
    }
    throw lastError!
}

/// Blocking wrapper around `reachableNodeConf()` for non-async tests.
func blockingReachableNodeConf() throws -> NodeConf {
    let semaphore = DispatchSemaphore(value: 0)
    var resolved: Result<NodeConf, Error>?
    Task {
        do {
            resolved = .success(try await reachableNodeConf())
        } catch {
            resolved = .failure(error)
        }
        semaphore.signal()
    }
    semaphore.wait()
    return try resolved!.get()
}
