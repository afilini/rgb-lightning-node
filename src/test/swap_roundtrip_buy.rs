use super::*;

const TEST_DIR_BASE: &str = "tmp/swaps/";
const NODE1_PEER_PORT: u16 = 9821;
const NODE2_PEER_PORT: u16 = 9822;
const NODE3_PEER_PORT: u16 = 9823;

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[traced_test]
async fn do_buy_swap() {
    initialize();

    let test_dir_node1 = format!("{TEST_DIR_BASE}node1");
    let test_dir_node2 = format!("{TEST_DIR_BASE}node2");
    let test_dir_node3 = format!("{TEST_DIR_BASE}node3");
    let (node1_addr, _) = start_node(test_dir_node1, NODE1_PEER_PORT, false).await;
    let (node2_addr, _) = start_node(test_dir_node2, NODE2_PEER_PORT, false).await;
    let (node3_addr, _) = start_node(test_dir_node3, NODE3_PEER_PORT, false).await;

    fund_and_create_utxos(node1_addr).await;
    fund_and_create_utxos(node2_addr).await;
    fund_and_create_utxos(node3_addr).await;

    let asset_id = issue_asset(node1_addr).await;

    let node2_info = node_info(node2_addr).await;
    let node2_pubkey = node2_info.pubkey;

    let node1_info = node_info(node1_addr).await;
    let node1_pubkey = node1_info.pubkey;

    open_colored_channel(node1_addr, &node2_pubkey, NODE2_PEER_PORT, 600, &asset_id).await;
    open_channel(node2_addr, &node1_pubkey, NODE2_PEER_PORT, 5000000, 546000).await;

    let maker_init_response =
        maker_init(node1_addr, 10, &asset_id, MakerInitSide::Buy, 3600, 5000).await;
    let taker_response = taker(node2_addr, maker_init_response.swapstring.clone()).await;

    dbg!(list_trades(node1_addr).await);
    dbg!(list_trades(node2_addr).await);

    maker_execute(
        node1_addr,
        maker_init_response.swapstring,
        maker_init_response.payment_secret,
        taker_response.our_pubkey,
    )
    .await;

    wait_for_ln_balance(node2_addr, &asset_id, 10).await;
}
