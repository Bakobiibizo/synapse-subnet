use super::*;
use crate::stake::{
    StakeError, StakeManager, StakeManagerImpl, StakeTransactionStatus, StakeTransactionType,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::test;

#[tokio::test]
async fn test_stake_manager_creation() {
    let stake_manager = StakeManagerImpl::new(1000, true);
    let info = stake_manager.get_stake_info().await.unwrap();
    assert_eq!(info.amount, 1000);
    assert!(info.auto_restake);
    assert_eq!(info.rewards_earned, 0);
}

#[tokio::test]
async fn test_stake_operations() {
    let mut stake_manager = StakeManagerImpl::new(1000, false);

    // Test staking
    let tx = stake_manager.stake(500).await.unwrap();
    assert_eq!(tx.amount, 500);
    assert_eq!(tx.transaction_type, StakeTransactionType::Stake);
    assert_eq!(tx.status, StakeTransactionStatus::Confirmed);

    // Verify updated stake amount
    let info = stake_manager.get_stake_info().await.unwrap();
    assert_eq!(info.amount, 1500);

    // Test unstaking
    let tx = stake_manager.unstake(300).await.unwrap();
    assert_eq!(tx.amount, 300);
    assert_eq!(tx.transaction_type, StakeTransactionType::Unstake);

    // Verify updated stake amount
    let info = stake_manager.get_stake_info().await.unwrap();
    assert_eq!(info.amount, 1200);
}

#[tokio::test]
async fn test_auto_restake() {
    let mut stake_manager = StakeManagerImpl::new(1000, true);

    // Simulate earning rewards
    stake_manager.get_stake_info().await.unwrap();

    // Test restaking
    let tx = stake_manager.restake().await.unwrap();
    assert_eq!(tx.amount, 100);
    assert_eq!(tx.transaction_type, StakeTransactionType::Restake);

    // Verify updated stake amount and cleared rewards
    let info = stake_manager.get_stake_info().await.unwrap();
    assert_eq!(info.amount, 1100);
    assert_eq!(info.rewards_earned, 0);
    assert!(info.last_restake.is_some());
}

#[tokio::test]
async fn test_claim_rewards() {
    let mut stake_manager = StakeManagerImpl::new(1000, false);

    // Simulate earning rewards
    stake_manager.get_stake_info().await.unwrap();

    // Test claiming rewards
    let tx = stake_manager.claim_rewards().await.unwrap();
    assert_eq!(tx.amount, 100);
    assert_eq!(tx.transaction_type, StakeTransactionType::RewardClaim);

    // Verify cleared rewards
    let info = stake_manager.get_stake_info().await.unwrap();
    assert_eq!(info.rewards_earned, 0);
}

#[tokio::test]
async fn test_stake_verification() {
    let stake_manager = StakeManagerImpl::new(1000, false);

    // Test stake verification
    assert!(stake_manager.verify_stake().await.unwrap());
}

#[tokio::test]
async fn test_invalid_stake_operations() {
    let mut stake_manager = StakeManagerImpl::new(1000, false);

    // Test unstaking more than available
    let result = stake_manager.unstake(1500).await;
    assert!(matches!(result, Err(StakeError::InsufficientStake(_))));

    // Test invalid stake amount
    let result = stake_manager.stake(0).await;
    assert!(matches!(result, Err(StakeError::InvalidStakeAmount(_))));

    // Test claiming rewards when none available
    let result = stake_manager.claim_rewards().await;
    assert!(matches!(result, Err(StakeError::OperationFailed(_))));
}

#[tokio::test]
async fn test_concurrent_operations() {
    let stake_manager = Arc::new(Mutex::new(StakeManagerImpl::new(1000, false)));

    // Simulate concurrent stake operations
    let stake_manager_clone = stake_manager.clone();
    let stake_fut1 = async move {
        let mut guard = stake_manager_clone.lock().await;
        guard.stake(500).await
    };

    let stake_manager_clone = stake_manager.clone();
    let stake_fut2 = async move {
        let mut guard = stake_manager_clone.lock().await;
        guard.stake(300).await
    };

    let (result1, result2) = tokio::join!(stake_fut1, stake_fut2);
    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Verify final stake amount
    let guard = stake_manager.lock().await;
    let info = guard.get_stake_info().await.unwrap();
    assert_eq!(info.amount, 1800);
}
