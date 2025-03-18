=========
Examples
=========

Generate mnemonic and address from secret key
=============================================

    .. testcode::

        from ergo_lib_python.wallet import MnemonicGenerator, ExtSecretKey, DerivationPath
        from ergo_lib_python.chain import NetworkPrefix
        mnemonic = MnemonicGenerator("english", 128).generate()
        # create Extended Secret Key and derive first secret key using default derivation path
        ext_secret_key = ExtSecretKey.from_mnemonic(mnemonic, password="").derive(DerivationPath())
        address = ext_secret_key.public_key().address()
        address.to_str(NetworkPrefix.Mainnet) # 9eg...

Build transaction
===================
    .. code-block:: python

        from ergo_lib_python.chain import ErgoBoxCandidate
        from ergo_lib_python.transaction import TxBuilder
        from ergo_lib_python.wallet import select_boxes_simple
        # Create a new output candidate with 1 billion nanoErgs (1 ERG)
        output_candidate = ErgoBoxCandidate(
            value=10 ** 9,
            script = Address("9egnPnrYskFS8k1gYiKZEXZ2bhP9fvX9GZvsG1V3BzH3n8sBXrf"),
            creation_height = 10000)
        boxes = [] # List of boxes belonging to signer, left empty here
        fee = 10**7 # Pay 0.01 ERG fee
        # Select boxes whose value sums up to amount sent to recipient (1 ERG) and miner fee (0.01 ERG)
        selection = select_boxes_simple(boxes, target_balance=output_candidate.value + fee, target_tokens=[])
        tx = TxBuilder(box_selection=selection,
                       output_candidates=[output_candidate],
                       current_height=1000,
                       fee_amount=fee,
                       change_address=Address("....")).build() # UnsignedTransaction
Mint token
=============
    .. code-block:: python

        from ergo_lib_python.chain import Token, TokenId, ErgoBoxCandidate, TxBuilder
        boxes = [] # list of boxes belonging to signer
        fee = 10**7 # Pay 0.01 ERG fee
        selection = select_boxes_simple(boxes, target_balance=10**9 + fee, [])
        # The identifier of the token must be the ID of the first input in the transaction
        mint_token = Token(TokenId(selection.boxes[0].box_id), amount=1)

        mint_candidate = ErgoBoxCandidate(
            value = 10**9,
            script = Address("...."),
            creation_height = 10000,
            mint_token = mint_token,
            mint_token_name = "NFT",
            mint_token_desc = "desc",
            mint_token_decimals = 0
        )
        tx = TxBuilder(
            box_selection=selection,
            output_candidates=[output_candidate],
            current_height=1000,
            fee_amount=fee,
            change_address=Address("....")).build() # UnsignedTransaction
