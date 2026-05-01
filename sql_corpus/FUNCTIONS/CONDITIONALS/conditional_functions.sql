-- Conditional Function Test Cases
-- Compatibility: MySQL 5.7+

SELECT IF(1 > 0, 'yes', 'no');

SELECT IFNULL(NULL, 'default');

SELECT IFNULL('value', 'default');

SELECT NULLIF(1, 1);

SELECT NULLIF(1, 2);

SELECT COALESCE(NULL, NULL, 'first');

SELECT COALESCE(NULL, 'second', 'third');

SELECT IF(price > 100, 'expensive', 'cheap') FROM products;

SELECT name, IF(age > 65, 'senior', IF(age > 18, 'adult', 'minor')) FROM users;

SELECT IFNULL(phone, 'no phone') FROM users;

SELECT NULLIF(price, 0) FROM products;

SELECT COALESCE(email, phone, 'no contact') FROM users;

SELECT name, CASE WHEN price > 100 THEN 'expensive' WHEN price > 50 THEN 'medium' ELSE 'cheap' END FROM products;

SELECT CASE status WHEN 'pending' THEN 1 WHEN 'active' THEN 2 WHEN 'completed' THEN 3 ELSE 0 END FROM orders;

SELECT name, CASE WHEN stock = 0 THEN 'out of stock' WHEN stock < 10 THEN 'low stock' WHEN stock < 50 THEN 'medium stock' ELSE 'in stock' END FROM products;

SELECT IF(COUNT(*) > 0, 'has orders', 'no orders') FROM orders WHERE user_id = 1;

SELECT COALESCE(SUM(total), 0) FROM orders WHERE user_id = 1;

SELECT IF(AVG(price) > 50, 'above average', 'below average') FROM products;

SELECT NULLIF(COUNT(*), 0) FROM products WHERE category_id = 1;

SELECT IF(MIN(price) > 0, 'has affordable', 'all expensive') FROM products;

SELECT IF(MAX(price) < 1000, 'budget friendly', 'premium') FROM products;

SELECT name, IF(price IS NULL, 'price missing', CAST(price AS CHAR)) FROM products;

SELECT IF(LENGTH(name) > 20, CONCAT(LEFT(name, 17), '...'), name) FROM users;

SELECT CASE WHEN email LIKE '%@company.com' THEN 'internal' WHEN email LIKE '%@partner.com' THEN 'partner' ELSE 'external' END FROM users;

SELECT COALESCE(category, 'uncategorized') FROM products;

SELECT IF(stock > 0, 'available', 'unavailable') FROM products WHERE id = 1;

SELECT NULLIF(LENGTH(TRIM(name)), 0) FROM users;

SELECT IF(YEAR(created_at) = 2024, 'current year', 'prior year') FROM orders;

SELECT CASE WHEN total > 1000 THEN 'high value' WHEN total > 500 THEN 'medium value' WHEN total > 0 THEN 'low value' ELSE 'no value' END FROM orders;

SELECT COALESCE(last_login, created_at) FROM users;

SELECT IF(status IN ('completed', 'shipped'), 'fulfilled', 'pending') FROM orders;

SELECT IFNULL(default_value, 'N/A') FROM settings;

SELECT NULLIF(price, original_price) FROM products;

SELECT CASE WHEN discount > 0 THEN price - discount ELSE price END FROM products;

SELECT IF(COUNT(DISTINCT status) > 1, 'mixed', 'consistent') FROM orders GROUP BY user_id;

SELECT COALESCE(MAX(total), 0) FROM orders WHERE user_id = 1;

SELECT IF(AVG(quantity) > 5, 'high volume', 'low volume') FROM order_items;

SELECT IFNULL(SUM(revenue), 0) FROM (SELECT SUM(price * quantity) AS revenue FROM order_items) t;

SELECT CASE WHEN stock <= 0 THEN 'out of stock' WHEN stock < reorder_level THEN 'reorder needed' ELSE 'in stock' END FROM inventory;

SELECT IF(pricing_mode = 'auto', 'automatic', 'manual') FROM products;

SELECT COALESCE(related_product_id, id) FROM products;

SELECT IFNULL(shipping_address, billing_address) FROM orders;

SELECT CASE WHEN payment_method = 'credit' THEN 'card' WHEN payment_method = 'debit' THEN 'card' ELSE 'other' END FROM transactions;

SELECT IF(created_at > updated_at, 'needs update', 'up to date') FROM records;

SELECT IFNULL(primary_contact, secondary_contact) FROM companies;

SELECT NULLIF(weight, 0) FROM products;

SELECT CASE WHEN rating >= 4.5 THEN 'excellent' WHEN rating >= 4.0 THEN 'good' WHEN rating >= 3.0 THEN 'average' ELSE 'poor' END FROM reviews;

SELECT IF(stock < 10, 'low', IF(stock < 50, 'medium', 'high')) FROM products;

SELECT COALESCE(CONVERT_TZ(created_at, 'UTC', 'America/New_York'), created_at) FROM orders;

SELECT IF(country IN ('USA', 'Canada', 'Mexico'), 'North America', 'Other') FROM users;

SELECT CASE status WHEN 'active' THEN 1 WHEN 'inactive' THEN 0 ELSE -1 END FROM users;

SELECT IFNULL(birth_date, DATE_SUB(CURDATE(), INTERVAL 30 YEAR)) FROM users;

SELECT IF(LENGTH(phone) >= 10, 'valid', 'invalid') FROM users;

SELECT COALESCE(department, 'General') FROM employees;

SELECT NULLIF(attempts, 0) FROM login_attempts;

SELECT CASE WHEN price * quantity > 1000 THEN 'wholesale' WHEN price * quantity > 100 THEN 'retail' ELSE 'small' END FROM order_items;

SELECT IF(CHAR_LENGTH(description) > 100, 'detailed', 'brief') FROM products;

SELECT IFNULL(preferred_language, 'en') FROM users;

SELECT IF(status = 'completed' AND total > 0, 'valid sale', 'invalid') FROM orders;

SELECT NULLIF(SUM(discount), 0) FROM orders WHERE user_id = 1;

SELECT CASE WHEN shipped_at IS NULL THEN 'not shipped' WHEN delivered_at IS NULL THEN 'in transit' ELSE 'delivered' END FROM orders;

SELECT COALESCE(NULLIF(notes, ''), 'no notes') FROM orders;

SELECT IF(YEARWEEK(created_at) = YEARWEEK(NOW()), 'this week', 'earlier') FROM orders;

SELECT IF(price BETWEEN 10 AND 50, 'budget', IF(price BETWEEN 51 AND 100, 'mid-range', 'premium')) FROM products;

SELECT CASE priority WHEN 1 THEN 'low' WHEN 2 THEN 'medium' WHEN 3 THEN 'high' ELSE 'urgent' END FROM tasks;

SELECT COALESCE(reference_id, id) FROM documents;

SELECT IFNULL(custom_field, default_field) FROM settings;

SELECT NULLIF(age, -1) FROM users;

SELECT CASE WHEN views > 10000 THEN 'viral' WHEN views > 1000 THEN 'popular' WHEN views > 100 THEN 'moderate' ELSE 'low' END FROM articles;

SELECT IF(stock >= order_quantity, 'fulfillable', 'insufficient') FROM products;

SELECT COALESCE(last_order_date, created_at) FROM users;

SELECT IF(COUNT(*) > 0, 'exists', 'not found') FROM users WHERE id = 999999;

SELECT NULLIF(total_weight, 0) FROM shipments;

SELECT CASE WHEN shipping_cost > 0 THEN 'paid shipping' ELSE 'free shipping' END FROM orders;

SELECT IF(email_verified = 1, 'verified', 'unverified') FROM users;

SELECT COALESCE(profile_complete, FALSE) FROM users;

SELECT IF(two_factor_enabled = 1, '2FA enabled', '2FA disabled') FROM users;

SELECT NULLIF(failed_attempts, 0) FROM login_attempts;

SELECT CASE WHEN subscription_status = 'active' THEN 'subscribed' WHEN subscription_status = 'expired' THEN 'expired' ELSE 'none' END FROM users;

SELECT IF(moderation_status = 'approved', 'published', 'pending') FROM posts;

SELECT IFNULL(last_activity, last_login) FROM users;

SELECT NULLIF(likes_count, 0) FROM posts;

SELECT CASE WHEN plan_type = 'free' THEN 0 WHEN plan_type = 'basic' THEN 10 WHEN plan_type = 'pro' THEN 50 ELSE 100 END FROM subscriptions;

SELECT IF(password_changed_at > DATE_SUB(NOW(), INTERVAL 90 DAY), 'password fresh', 'password expiring') FROM users;

SELECT COALESCE(api_key, oauth_token) FROM integrations;

SELECT NULLIF(session_token, '') FROM sessions;

SELECT IF(is_verified = 1 AND is_active = 1, 'fully active', 'restricted') FROM users;

SELECT CASE WHEN account_type = 'admin' THEN 'administrator' WHEN account_type = 'moderator' THEN 'moderator' ELSE 'user' END FROM users;

SELECT IFNULL(default_theme, 'light') FROM user_settings;

SELECT NULLIF(custom_css, '') FROM themes;

SELECT CASE WHEN notification_email = 1 THEN 'email on' ELSE 'email off' END FROM user_preferences;

SELECT IF(autorenew_enabled = 1, 'will renew', 'will expire') FROM subscriptions;

SELECT COALESCE(billing_address, shipping_address) FROM users;

SELECT IF(marketing_opt_in = 1, 'opted in', 'opted out') FROM users;

SELECT NULLIF(referral_code, '') FROM users;

SELECT CASE WHEN user_type = 'premium' THEN 'premium user' ELSE 'standard user' END FROM users;

SELECT IFNULL(avatar_url, '/default-avatar.png') FROM users;

SELECT IF(banned = 1, 'banned', 'active') FROM users;

SELECT NULLIF(warning_count, 0) FROM users;

SELECT CASE WHEN trust_level = 5 THEN 'expert' WHEN trust_level >= 3 THEN 'trusted' ELSE 'new' END FROM users;

SELECT IF(email_bounce = 1, 'bounced', 'valid') FROM users;

SELECT COALESCE(timezone, 'UTC') FROM users;

SELECT IF(privacy_public = 1, 'public', 'private') FROM user_profiles;

SELECT NULLIF(profile_views, 0) FROM user_stats;

SELECT CASE WHEN subscription_tier = 'gold' THEN 'gold member' WHEN subscription_tier = 'silver' THEN 'silver member' ELSE 'standard' END FROM users;

SELECT IF(allow_messages = 1, 'messages allowed', 'messages blocked') FROM user_settings;

SELECT COALESCE(last_purchase_date, created_at) FROM users;

SELECT IFNULL(custom_status, 'available') FROM user_status;

SELECT NULLIF(credits_remaining, 0) FROM user_accounts;

SELECT CASE WHEN membership_level = 'platinum' THEN 3 WHEN membership_level = 'gold' THEN 2 WHEN membership_level = 'silver' THEN 1 ELSE 0 END FROM users;

SELECT IF(show_online_status = 1, 'visible', 'hidden') FROM user_settings;

SELECT COALESCE(fallback_value, 0) FROM calculations;

SELECT IFNULL(language, 'en-US') FROM locales;

SELECT NULLIF(rating_count, 0) FROM products;

SELECT CASE WHEN delivery_status = 'delivered' THEN 3 WHEN delivery_status = 'shipped' THEN 2 WHEN delivery_status = 'pending' THEN 1 ELSE 0 END FROM orders;

SELECT IF(account_balance > 0, 'credit', 'debit') FROM accounts;

SELECT COALESCE(company_name, CONCAT(first_name, ' ', last_name)) FROM contacts;

SELECT IFNULL(rating, 0) FROM reviews;

SELECT NULLIF(response_time, 0) FROM support_tickets;

SELECT CASE WHEN resolution_status = 'resolved' THEN 3 WHEN resolution_status = 'in_progress' THEN 2 ELSE 1 END FROM support_tickets;

SELECT IF(is_critical = 1, 'CRITICAL', 'normal') FROM alerts;

SELECT COALESCE(sla_tier, 'standard') FROM customer_accounts;

SELECT IFNULL(redirect_url, '/home') FROM auth_sessions;

SELECT NULLIF(import_batch_id, 0) FROM imported_records;

SELECT CASE WHEN import_status = 'complete' THEN 1 WHEN import_status = 'partial' THEN 0 ELSE -1 END FROM imports;

SELECT IF(sync_required = 1, 'needs sync', 'synced') FROM external_integrations;

SELECT COALESCE(encryption_key, 'default_key') FROM secure_storage;

SELECT IFNULL(master_pin, user_pin) FROM authentication;

SELECT NULLIF(access_token, '') FROM oauth_sessions;

SELECT CASE WHEN permission_level >= 5 THEN 'full access' WHEN permission_level >= 3 THEN 'limited access' ELSE 'minimal access' END FROM permissions;

SELECT IF(is_encrypted = 1, 'encrypted', 'plaintext') FROM sensitive_data;

SELECT COALESCE(signed_by, 'unsigned') FROM documents;

SELECT IFNULL(valid_from, created_at) FROM certificates;

SELECT NULLIF(valid_until, '9999-12-31') FROM certificates;

SELECT CASE WHEN certificate_status = 'valid' THEN 1 WHEN certificate_status = 'expired' THEN 0 ELSE -1 END FROM certificates;

SELECT IF(uses_two_factor = 1, '2FA required', '2FA optional') FROM security_settings;

SELECT COALESCE(ip_whitelist, 'any') FROM access_control;

SELECT NULLIF(session_timeout, 0) FROM security_settings;

SELECT CASE WHEN security_level = 'high' THEN 3 WHEN security_level = 'medium' THEN 2 ELSE 1 END FROM security_settings;

SELECT IF(is_locked = 1, 'locked', 'unlocked') FROM security_locks;

SELECT IFNULL(backup_frequency, 'daily') FROM backup_settings;

SELECT NULLIF(last_backup, '1970-01-01') FROM backup_logs;

SELECT CASE WHEN backup_status = 'success' THEN 1 WHEN backup_status = 'failed' THEN 0 ELSE -1 END FROM backup_logs;

SELECT IF(auto_backup = 1, 'auto backup on', 'auto backup off') FROM settings;

SELECT COALESCE(storage_quota, 10737418240) FROM storage_settings;

SELECT NULLIF(used_storage, 0) FROM storage_stats;

SELECT IF(storage_usage > 80, 'storage warning', 'storage ok') FROM storage_stats;

SELECT CASE WHEN file_version > 1 THEN 'versioned' ELSE 'original' END FROM document_versions;

SELECT IFNULL(checkout_token, session_token) FROM transactions;

SELECT NULLIF(authorization_code, '') FROM oauth_flows;

SELECT CASE WHEN token_type = 'bearer' THEN 'Bearer auth' WHEN token_type = 'basic' THEN 'Basic auth' ELSE 'Other' END FROM auth_tokens;

SELECT IF(is_revoked = 1, 'revoked', 'active') FROM auth_tokens;

SELECT COALESCE(refresh_token, access_token) FROM token_pairs;

SELECT IFNULL(id_token, access_token) FROM auth_responses;

SELECT NULLIF(auth_method, 'none') FROM auth_sessions;

SELECT CASE WHEN auth_provider = 'google' THEN 'Google OAuth' WHEN auth_provider = 'github' THEN 'GitHub OAuth' ELSE 'Local' END FROM auth_sessions;

SELECT IF(is_anonymous = 1, 'anonymous', 'identified') FROM sessions;

SELECT COALESCE(user_agent, 'unknown') FROM sessions;

SELECT NULLIF(ip_address, '') FROM sessions;

SELECT IF(is_botnet = 1, 'suspicious', 'clean') FROM session_logs;

SELECT COALESCE(country_code, 'XX') FROM geoip_data;

SELECT NULLIF(asn_number, 0) FROM network_info;

SELECT CASE WHEN connection_type = 'mobile' THEN 'Mobile' WHEN connection_type = 'broadband' THEN 'Fixed Line' ELSE 'Other' END FROM connection_logs;

SELECT IFNULL(latitude, 0.0) FROM location_data;

SELECT NULLIF(longitude, 0.0) FROM location_data;

SELECT CASE WHEN accuracy < 100 THEN 'high accuracy' WHEN accuracy < 1000 THEN 'medium accuracy' ELSE 'low accuracy' END FROM location_data;

SELECT IF(is_within_radius = 1, 'within zone', 'outside zone') FROM geofence_events;

SELECT COALESCE(nearest_landmark, 'unknown') FROM location_cache;

SELECT IFNULL(timezone, 'UTC') FROM timezone_data;

SELECT NULLIF(dst_offset, 0) FROM timezone_data;

SELECT CASE WHEN altitude > 5000 THEN 'high altitude' WHEN altitude > 1000 THEN 'medium altitude' ELSE 'low altitude' END FROM elevation_data;

SELECT IFNULL(speed, 0) FROM gps_tracking;

SELECT NULLIF(bearing, -1) FROM compass_data;

SELECT CASE WHEN battery_level > 80 THEN 'high battery' WHEN battery_level > 20 THEN 'medium battery' ELSE 'low battery' END FROM device_status;

SELECT IF(is_charging = 1, 'charging', 'on battery') FROM device_status;

SELECT COALESCE(signal_strength, -1) FROM network_metrics;

SELECT NULLIF(latency_ms, 0) FROM network_metrics;

SELECT CASE WHEN packet_loss > 10 THEN 'poor connection' WHEN packet_loss > 1 THEN 'fair connection' ELSE 'good connection' END FROM network_metrics;

SELECT IFNULL(jitter_ms, 0) FROM network_metrics;

SELECT NULLIF(bandwidth_mbps, 0) FROM network_metrics;

SELECT CASE WHEN bandwidth > 100 THEN 'fast' WHEN bandwidth > 25 THEN 'average' ELSE 'slow' END FROM connection_quality;

SELECT IF(is_online = 1, 'online', 'offline') FROM device_status;

SELECT COALESCE(last_heartbeat, 'never') FROM device_status;

SELECT NULLIF(error_count, 0) FROM device_logs;

SELECT CASE WHEN uptime_hours > 8760 THEN '>1 year' WHEN uptime_hours > 720 THEN '>1 month' ELSE '<1 month' END FROM system_status;

SELECT IFNULL(cpu_usage, 0) FROM system_metrics;

SELECT NULLIF(memory_usage, 0) FROM system_metrics;

SELECT CASE WHEN memory_usage > 90 THEN 'memory critical' WHEN memory_usage > 70 THEN 'memory warning' ELSE 'memory ok' END FROM system_metrics;

SELECT IFNULL(disk_usage, 0) FROM system_metrics;

SELECT NULLIF(disk_io_wait, 0) FROM system_metrics;

SELECT CASE WHEN disk_usage > 95 THEN 'disk critical' WHEN disk_usage > 85 THEN 'disk warning' ELSE 'disk ok' END FROM disk_metrics;

SELECT COALESCE(network_in, 0) FROM network_metrics;

SELECT NULLIF(network_out, 0) FROM network_metrics;

SELECT IF(is_primary = 1, 'primary', 'secondary') FROM network_interfaces;

SELECT CASE WHEN interface_status = 'up' THEN 1 ELSE 0 END FROM network_interfaces;

SELECT IFNULL(process_count, 0) FROM system_status;

SELECT NULLIF(thread_count, 0) FROM system_status;

SELECT CASE WHEN load_average > 10 THEN 'high load' WHEN load_average > 5 THEN 'medium load' ELSE 'low load' END FROM system_status;

SELECT IFNULL(temperature_celsius, 0) FROM hardware_sensors;

SELECT NULLIF(fan_speed_rpm, 0) FROM hardware_sensors;

SELECT CASE WHEN temperature_celsius > 90 THEN 'thermal critical' WHEN temperature_celsius > 80 THEN 'thermal warning' ELSE 'thermal ok' END FROM hardware_sensors;

SELECT IFNULL(voltage, 0) FROM power_supply;

SELECT NULLIF(power_watts, 0) FROM power_consumption;

SELECT CASE WHEN power_watts > 500 THEN 'high power' WHEN power_watts > 100 THEN 'medium power' ELSE 'low power' END FROM power_consumption;

SELECT COALESCE(energy_kwh, 0) FROM power_usage;

SELECT NULLIF(cost, 0) FROM billing;

SELECT IF(is_billable = 1, 'billable', 'non-billable') FROM usage_records;

SELECT CASE WHEN billing_cycle = 'monthly' THEN 1 WHEN billing_cycle = 'quarterly' THEN 3 ELSE 12 END FROM billing_settings;

SELECT IFNULL(discount_percent, 0) FROM pricing;

SELECT NULLIF(tax_rate, 0) FROM tax_settings;

SELECT CASE WHEN payment_status = 'paid' THEN 1 WHEN payment_status = 'pending' THEN 0 ELSE -1 END FROM invoices;

SELECT COALESCE(amount_due, amount_total) FROM invoices;

SELECT IFNULL(amount_paid, 0) FROM invoices;

SELECT NULLIF(overdue_days, 0) FROM invoices;

SELECT CASE WHEN overdue_days > 90 THEN 'bad debt' WHEN overdue_days > 30 THEN 'delinquent' ELSE 'current' END FROM accounts_receivable;

SELECT IF(credit_limit > 0, 'has credit', 'no credit') FROM customer_accounts;

SELECT COALESCE(available_credit, credit_limit) FROM credit_limits;

SELECT NULLIF(interest_rate, 0) FROM loan_accounts;

SELECT CASE WHEN loan_status = 'active' THEN 1 WHEN loan_status = 'paid_off' THEN 0 ELSE -1 END FROM loan_accounts;

SELECT IFNULL(principal, 0) FROM amortization;

SELECT NULLIF(payment_number, 0) FROM payment_schedule;

SELECT CASE WHEN payment_status = 'scheduled' THEN 1 WHEN payment_status = 'processed' THEN 2 ELSE 0 END FROM payment_schedule;

SELECT COALESCE(amount, 0) FROM payments;

SELECT NULLIF(transaction_fee, 0) FROM transactions;

SELECT CASE WHEN transaction_type = 'credit' THEN 1 WHEN transaction_type = 'debit' THEN -1 ELSE 0 END FROM transactions;

SELECT IF(is_reconciled = 1, 'reconciled', 'pending') FROM transactions;

SELECT COALESCE(check_number, 'N/A') FROM checks;

SELECT NULLIF(routing_number, '') FROM bank_accounts;

SELECT IFNULL(account_number, 'XXXX') FROM bank_accounts;

SELECT CASE WHEN account_status = 'active' THEN 1 WHEN account_status = 'closed' THEN 0 ELSE -1 END FROM bank_accounts;

SELECT IF(is_verified = 1, 'verified', 'unverified') FROM payment_methods;

SELECT COALESCE(card_last_four, 'XXXX') FROM payment_methods;

SELECT NULLIF(expiry_month, 0) FROM payment_methods;

SELECT CASE WHEN card_type = 'credit' THEN 'credit card' WHEN card_type = 'debit' THEN 'debit card' ELSE 'prepaid' END FROM payment_methods;

SELECT IF(is_default = 1, 'default', 'secondary') FROM payment_methods;

SELECT COALESCE(CONCAT('****', last_four), 'no card') FROM stored_cards;

SELECT NULLIF(auth_code, '') FROM authorizations;

SELECT CASE WHEN auth_status = 'approved' THEN 1 WHEN auth_status = 'declined' THEN 0 ELSE -1 END FROM authorizations;

SELECT IFNULL(remaining_authorization, 0) FROM authorizations;

SELECT COALESCE(captured_amount, 0) FROM captures;

SELECT NULLIF(refunded_amount, 0) FROM refunds;

SELECT CASE WHEN refund_status = 'full' THEN 1 WHEN refund_status = 'partial' THEN 0 ELSE -1 END FROM refunds;

SELECT IF(is_settled = 1, 'settled', 'pending') FROM transactions;

SELECT COALESCE(settlement_date, 'unsettled') FROM transactions;

SELECT NULLIF(chargeback_amount, 0) FROM disputes;

SELECT CASE WHEN dispute_status = 'won' THEN 1 WHEN dispute_status = 'lost' THEN 0 ELSE -1 END FROM disputes;

SELECT IFNULL(dispute_reason, 'N/A') FROM disputes;

SELECT COALESCE(arbitration_decision, 'pending') FROM disputes;

SELECT NULLIF(escrow_amount, 0) FROM escrow_accounts;

SELECT CASE WHEN escrow_status = 'funded' THEN 1 WHEN escrow_status = 'released' THEN 2 ELSE 0 END FROM escrow_accounts;

SELECT IFNULL(release_date, 'held') FROM escrow_releases;

SELECT COALESCE(buyer_protection, FALSE) FROM transactions;

SELECT NULLIF(seller_guarantee, FALSE) FROM seller_accounts;

SELECT CASE WHEN seller_rating >= 4.5 THEN 'top seller' WHEN seller_rating >= 4.0 THEN 'good seller' ELSE 'average seller' END FROM seller_metrics;

SELECT IFNULL(response_rate, 0) FROM seller_metrics;

SELECT NULLIF(completion_rate, 0) FROM seller_metrics;

SELECT CASE WHEN on_time_delivery > 95 THEN 'excellent' WHEN on_time_delivery > 90 THEN 'good' ELSE 'needs improvement' END FROM delivery_metrics;

SELECT COALESCE(return_rate, 0) FROM return_metrics;

SELECT NULLIF(exchange_rate, 1) FROM currency_conversions;

SELECT CASE WHEN currency = 'USD' THEN '$' WHEN currency = 'EUR' THEN '€' ELSE '¥' END FROM pricing;

SELECT IFNULL(local_price, 0) FROM regional_pricing;

SELECT NULLIF(international_shipping, 0) FROM shipping_options;

SELECT CASE WHEN shipping_method = 'express' THEN 1 WHEN shipping_method = 'standard' THEN 0 ELSE -1 END FROM shipping_options;

SELECT IF(is_tracking_included = 1, 'tracked', 'untracked') FROM shipments;

SELECT COALESCE(carrier, 'standard') FROM shipping_labels;

SELECT NULLIF(tracking_number, '') FROM tracking_events;

SELECT CASE WHEN delivery_status = 'delivered' THEN 3 WHEN delivery_status = 'in_transit' THEN 2 WHEN delivery_status = 'picked_up' THEN 1 ELSE 0 END FROM tracking_events;

SELECT IFNULL(delivery_timestamp, 'pending') FROM tracking_events;

SELECT COALESCE(signature, 'no signature') FROM delivery_confirmations;

SELECT NULLIF(proof_image, '') FROM delivery_proofs;

SELECT CASE WHEN delivery_attempt = 1 THEN 'first attempt' WHEN delivery_attempt = 2 THEN 'second attempt' ELSE 'final attempt' END FROM delivery_attempts;

SELECT IFNULL(return_reason, 'N/A') FROM returns;

SELECT NULLIF(restocking_fee, 0) FROM return_processing;

SELECT CASE WHEN return_status = 'approved' THEN 1 WHEN return_status = 'rejected' THEN 0 ELSE -1 END FROM returns;

SELECT COALESCE(refund_amount, 0) FROM return_refunds;

SELECT IFNULL(replacement_item, 'no replacement') FROM return_exchanges;

SELECT NULLIF(return_shipping_cost, 0) FROM return_costs;

SELECT CASE WHEN return_condition = 'new' THEN 3 WHEN return_condition = 'opened' THEN 2 ELSE 1 END FROM return_inspections;

SELECT IF(is_inspected = 1, 'inspected', 'pending inspection') FROM return_inspections;

SELECT COALESCE(inspector_notes, 'no notes') FROM return_inspections;

SELECT NULLIF(reinspection_count, 0) FROM quality_control;

SELECT CASE WHEN quality_grade = 'A' THEN 4 WHEN quality_grade = 'B' THEN 3 WHEN quality_grade = 'C' THEN 2 ELSE 1 END FROM quality_grades;

SELECT IFNULL(test_result, 'pending') FROM quality_tests;

SELECT NULLIF(defect_count, 0) FROM quality_reports;

SELECT CASE WHEN compliance_status = 'compliant' THEN 1 WHEN compliance_status = 'warning' THEN 0 ELSE -1 END FROM compliance_checks;

SELECT COALESCE(certification_number, 'none') FROM certifications;

SELECT NULLIF(expiry_date, '9999-12-31') FROM certifications;

SELECT CASE WHEN certification_level = 'gold' THEN 3 WHEN certification_level = 'silver' THEN 2 ELSE 1 END FROM certification_levels;

SELECT IFNULL(audit_score, 0) FROM audit_results;

SELECT NULLIF(audit_findings, 0) FROM audit_reports;

SELECT CASE WHEN audit_status = 'passed' THEN 1 WHEN audit_status = 'failed' THEN 0 ELSE -1 END FROM audit_results;

SELECT IFNULL(corrective_action, 'none') FROM audit_findings;

SELECT COALESCE(preventive_action, 'none') FROM audit_findings;

SELECT NULLIF(follow_up_date, '9999-12-31') FROM audit_findings;

SELECT CASE WHEN risk_level = 'high' THEN 3 WHEN risk_level = 'medium' THEN 2 ELSE 1 END FROM risk_assessments;

SELECT IFNULL(mitigation_plan, 'no plan') FROM risk_management;

SELECT NULLIF(residual_risk, 0) FROM risk_analysis;

SELECT CASE WHEN control_effectiveness = 'effective' THEN 3 WHEN control_effectiveness = 'partially_effective' THEN 2 ELSE 1 END FROM control_assessments;

SELECT COALESCE(policy_version, '1.0') FROM policy_documents;

SELECT NULLIF(review_frequency, 0) FROM policy_reviews;

SELECT IF(is_mandatory = 1, 'mandatory', 'optional') FROM training_modules;

SELECT NULLIF(completion_status, 'incomplete') FROM training_completions;

SELECT CASE WHEN assessment_score >= 80 THEN 'passed' WHEN assessment_score >= 60 THEN 'conditional' ELSE 'failed' END FROM assessment_results;

SELECT IFNULL(certificate_issued, 'pending') FROM training_records;

SELECT NULLIF(continuing_education_units, 0) FROM professional_development;

SELECT COALESCE(license_number, 'none') FROM professional_licenses;

SELECT NULLIF(license_expiry, '9999-12-31') FROM professional_licenses;

SELECT CASE WHEN renewal_status = 'renewed' THEN 1 WHEN renewal_status = 'pending' THEN 0 ELSE -1 END FROM license_renewals;

SELECT IFNULL(accreditation_body, 'internal') FROM accreditations;

SELECT NULLIF(accreditation_status, 'none') FROM accreditation_statuses;

SELECT CASE WHEN accreditation_level = 'full' THEN 3 WHEN accreditation_level = 'provisional' THEN 2 ELSE 1 END FROM accreditation_levels;

SELECT COALESCE(compliance_framework, 'internal') FROM compliance_frameworks;

SELECT NULLIF(control_count, 0) FROM compliance_controls;

SELECT CASE WHEN implementation_status = 'implemented' THEN 3 WHEN implementation_status = 'planned' THEN 2 ELSE 1 END FROM control_implementation;

SELECT IFNULL(effectiveness_rating, 0) FROM control_evaluations;

SELECT NULLIF(exception_count, 0) FROM control_exceptions;

SELECT CASE WHEN exception_status = 'approved' THEN 1 WHEN exception_status = 'denied' THEN 0 ELSE -1 END FROM control_exceptions;

SELECT COALESCE(waiver_expiry, '9999-12-31') FROM control_waivers;

SELECT IFNULL(compensating_control, 'none') FROM risk_treatments;

SELECT NULLIF(treatment_cost, 0) FROM risk_treatments;

SELECT CASE WHEN treatment_effectiveness = 'effective' THEN 3 WHEN treatment_effectiveness = 'partial' THEN 2 ELSE 1 END FROM treatment_effectiveness;

SELECT COALESCE(incident_count, 0) FROM security_metrics;

SELECT NULLIF(breach_count, 0) FROM security_incidents;

SELECT CASE WHEN incident_severity = 'critical' THEN 4 WHEN incident_severity = 'high' THEN 3 WHEN incident_severity = 'medium' THEN 2 ELSE 1 END FROM security_incidents;

SELECT IFNULL(containment_time, 0) FROM incident_responses;

SELECT NULLIF(eradication_time, 0) FROM incident_responses;

SELECT CASE WHEN recovery_time < 1 THEN 'excellent' WHEN recovery_time < 4 THEN 'good' ELSE 'needs improvement' END FROM incident_recovery;

SELECT COALESCE(total_downtime, 0) FROM incident_impact;

SELECT NULLIF(affected_users, 0) FROM incident_impact;

SELECT IF(is_published = 1, 'published', 'draft') FROM incident_reports;

SELECT COALESCE(lessons_learned, 'none') FROM incident_postmortems;

SELECT NULLIF(prevention_measures, 'none') FROM incident_prevention;

SELECT CASE WHEN root_cause = 'process' THEN 1 WHEN root_cause = 'technology' THEN 2 WHEN root_cause = 'human' THEN 3 ELSE 0 END FROM root_cause_analysis;

SELECT IFNULL(recurrence_likelihood, 'unknown') FROM risk_predictions;

SELECT NULLIF(predicted_impact, 0) FROM risk_predictions;

SELECT CASE WHEN risk_score > 80 THEN 'critical' WHEN risk_score > 60 THEN 'high' WHEN risk_score > 40 THEN 'medium' ELSE 'low' END FROM risk_scores;

SELECT COALESCE(recommended_action, 'monitor') FROM risk_recommendations;

SELECT NULLIF(action_priority, 0) FROM recommended_actions;

SELECT CASE WHEN action_status = 'completed' THEN 1 WHEN action_status = 'in_progress' THEN 0 ELSE -1 END FROM action_tracking;

SELECT IFNULL(completion_date, 'pending') FROM action_tracking;

SELECT COALESCE(responsible_party, 'unassigned') FROM action_assignments;

SELECT NULLIF(budget_allocated, 0) FROM action_budgets;

SELECT CASE WHEN budget_status = 'approved' THEN 1 WHEN budget_status = 'pending' THEN 0 ELSE -1 END FROM action_budgets;

SELECT IFNULL(resource_count, 0) FROM resource_allocations;

SELECT NULLIF(tool_requirement, 'none') FROM resource_requirements;

SELECT CASE WHEN resource_status = 'available' THEN 1 WHEN resource_status = 'allocated' THEN 0 ELSE -1 END FROM resource_status;

SELECT COALESCE(skill_level, 'novice') FROM skill_assessments;

SELECT NULLIF(certification_level, 'none') FROM skill_certifications;

SELECT CASE WHEN proficiency >= 4 THEN 'expert' WHEN proficiency >= 3 THEN 'proficient' WHEN proficiency >= 2 THEN 'competent' ELSE 'novice' END FROM proficiency_levels;

SELECT IFNULL(training_hours, 0) FROM training_records;

SELECT NULLIF(mentorship_hours, 0) FROM development_records;

SELECT COALESCE(performance_rating, 0) FROM performance_reviews;

SELECT NULLIF(goal_completion, 0) FROM performance_goals;

SELECT CASE WHEN rating_change > 0 THEN 'improved' WHEN rating_change < 0 THEN 'declined' ELSE 'maintained' END FROM performance_trends;

SELECT IFNULL(next_review_date, 'TBD') FROM performance_schedules;

SELECT COALESCE(feedback, 'no feedback') FROM performance_feedback;

SELECT NULLIF(development_plan, 'none') FROM career_development;

SELECT CASE WHEN potential_level = 'high' THEN 3 WHEN potential_level = 'medium' THEN 2 ELSE 1 END FROM potential_assessments;

SELECT IFNULL(successor, 'none') FROM succession_planning;

SELECT NULLIF(readiness_level, 0) FROM succession_readiness;

SELECT CASE WHEN readiness_status = 'ready' THEN 3 WHEN readiness_status = 'developing' THEN 2 ELSE 1 END FROM succession_status;

SELECT COALESCE(retention_risk, 'low') FROM retention_risks;

SELECT NULLIF(flight_risk, 0) FROM flight_risk_assessments;

SELECT CASE WHEN engagement_level = 'highly_engaged' THEN 4 WHEN engagement_level = 'engaged' THEN 3 WHEN engagement_level = 'disengaged' THEN 2 ELSE 1 END FROM engagement_surveys;

SELECT IFNULL(satisfaction_score, 0) FROM employee_satisfaction;

SELECT NULLIF(turnover_rate, 0) FROM turnover_metrics;

SELECT CASE WHEN absenteeism > 5 THEN 'high' WHEN absenteeism > 2 THEN 'moderate' ELSE 'low' END FROM absenteeism_rates;

SELECT COALESCE(workers_comp_claims, 0) FROM safety_metrics;

SELECT NULLIF(near_miss_count, 0) FROM safety_incidents;

SELECT CASE WHEN safety_rating >= 90 THEN 'excellent' WHEN safety_rating >= 75 THEN 'good' ELSE 'needs improvement' END FROM safety_ratings;

SELECT IFNULL(last_safety_training, 'none') FROM safety_compliance;

SELECT COALESCE(injury_count, 0) FROM workplace_incidents;

SELECT NULLIF(incident_investigation, 'pending') FROM incident_reports;

SELECT CASE WHEN osha_recordable = 1 THEN 'recordable' ELSE 'non-recordable' END FROM osha_records;

SELECT COALESCE(days_lost, 0) FROM productivity_metrics;

SELECT NULLIF(quality_defects, 0) FROM quality_metrics;

SELECT CASE WHEN efficiency > 90 THEN 'excellent' WHEN efficiency > 75 THEN 'good' ELSE 'needs improvement' END FROM efficiency_metrics;

SELECT IFNULL(capacity_utilization, 0) FROM capacity_planning;

SELECT NULLIF(bottleneck_count, 0) FROM process_analysis;

SELECT COALESCE(cycle_time, 0) FROM process_metrics;

SELECT NULLIF(throughput_rate, 0) FROM throughput_measurements;

SELECT CASE WHEN utilization > 85 THEN 'overutilized' WHEN utilization > 50 THEN 'optimal' ELSE 'underutilized' END FROM utilization_rates;

SELECT IFNULL(lead_time, 0) FROM supply_chain_metrics;

SELECT NULLIF(inventory_turnover, 0) FROM inventory_metrics;

SELECT CASE WHEN stockout_risk = 'high' THEN 3 WHEN stockout_risk = 'medium' THEN 2 ELSE 1 END FROM inventory_risks;

SELECT COALESCE(reorder_point, 0) FROM inventory_policies;

SELECT NULLIF(safety_stock, 0) FROM inventory_levels;

SELECT CASE WHEN demand_volatility = 'high' THEN 3 WHEN demand_volatility = 'medium' THEN 2 ELSE 1 END FROM demand_forecasting;

SELECT IFNULL(forecast_accuracy, 0) FROM forecast_metrics;

SELECT NULLIF(forecast_bias, 0) FROM forecast_analysis;

SELECT COALESCE(seasonal_factor, 1) FROM seasonal_adjustments;

SELECT NULLIF(trend_factor, 1) FROM trend_analysis;

SELECT CASE WHEN forecast_confidence > 90 THEN 'high confidence' WHEN forecast_confidence > 70 THEN 'medium confidence' ELSE 'low confidence' END FROM confidence_levels;

SELECT IFNULL(allocations_remaining, 0) FROM budget_allocations;

SELECT NULLIF(variance_percent, 0) FROM budget_variance;

SELECT CASE WHEN variance > 10 THEN 'over budget' WHEN variance < -10 THEN 'under budget' ELSE 'on budget' END FROM budget_performance;

SELECT COALESCE(budget_year, YEAR(CURDATE())) FROM fiscal_planning;

SELECT NULLIF(fiscal_quarter, 0) FROM quarterly_results;

SELECT CASE WHEN revenue_growth > 0 THEN 'growing' WHEN revenue_growth = 0 THEN 'stable' ELSE 'declining' END FROM growth_analysis;

SELECT IFNULL(profit_margin, 0) FROM profitability_metrics;

SELECT NULLIF(operating_margin, 0) FROM operational_efficiency;

SELECT COALESCE(return_on_investment, 0) FROM roi_calculations;

SELECT NULLIF(customer_acquisition_cost, 0) FROM acquisition_metrics;

SELECT CASE WHEN clv > cac * 3 THEN 'healthy' WHEN clv > cac THEN 'acceptable' ELSE 'unhealthy' END FROM customer_economics;

SELECT COALESCE(net_promoter_score, 0) FROM nps_surveys;

SELECT NULLIF(customer_effort_score, 0) FROM ces_metrics;

SELECT CASE WHEN ces_score > 8 THEN 'easy' WHEN ces_score > 5 THEN 'moderate' ELSE 'difficult' END FROM effort_scores;

SELECT IFNULL(first_response_time, 0) FROM service_metrics;

SELECT NULLIF(resolution_time, 0) FROM support_metrics;

SELECT COALESCE(first_contact_resolution, FALSE) FROM resolution_metrics;

SELECT CASE WHEN csat_score >= 4.5 THEN 'excellent' WHEN csat_score >= 4.0 THEN 'good' ELSE 'needs improvement' END FROM satisfaction_scores;

SELECT IFNULL(repeat_purchase_rate, 0) FROM loyalty_metrics;

SELECT NULLIF(average_order_value, 0) FROM customer_value_metrics;

SELECT COALESCE(customer_lifetime_value, 0) FROM clv_calculations;

SELECT NULLIF(churn_probability, 0) FROM churn_predictions;

SELECT CASE WHEN churn_risk = 'high' THEN 3 WHEN churn_risk = 'medium' THEN 2 ELSE 1 END FROM risk_classifications;

SELECT IFNULL(re_engagement_offer, 'none') FROM churn_prevention;

SELECT COALESCE(discount_offered, 0) FROM retention_offers;

SELECT NULLIF(loyalty_points, 0) FROM loyalty_programs;

SELECT CASE WHEN tier_status = 'platinum' THEN 4 WHEN tier_status = 'gold' THEN 3 WHEN tier_status = 'silver' THEN 2 ELSE 1 END FROM tier_statuses;

SELECT IFNULL(redemption_rate, 0) FROM reward_redemptions;

SELECT NULLIF(points_expiring, 0) FROM loyalty_expirations;

SELECT COALESCE(referral_bonus, 0) FROM referral_programs;

SELECT NULLIF(referral_count, 0) FROM referral_tracking;

SELECT CASE WHEN referral_quality = 'high' THEN 3 WHEN referral_quality = 'medium' THEN 2 ELSE 1 END FROM referral_quality;

SELECT IFNULL(advocacy_score, 0) FROM advocacy_metrics;

SELECT NULLIF(social_shares, 0) FROM social_engagement;

SELECT COALESCE(brand_mention_sentiment, 'neutral') FROM sentiment_tracking;

SELECT NULLIF(review_count, 0) FROM review_metrics;

SELECT CASE WHEN average_rating >= 4.5 THEN 'excellent' WHEN average_rating >= 4.0 THEN 'good' ELSE 'average' END FROM rating_analysis;

SELECT IFNULL(response_rate, 0) FROM review_responses;

SELECT NULLIF(response_time_hours, 0) FROM review_response_metrics;

SELECT COALESCE(positive_sentiment_percent, 0) FROM sentiment_analysis;

SELECT NULLIF(negative_sentiment_percent, 0) FROM sentiment_analysis;

SELECT CASE WHEN sentiment_trend = 'improving' THEN 1 WHEN sentiment_trend = 'stable' THEN 0 ELSE -1 END FROM sentiment_trends;

SELECT IFNULL(share_of_voice, 0) FROM competitive_metrics;

SELECT NULLIF(mention_volume, 0) FROM media_monitoring;

SELECT COALESCE(influence_score, 0) FROM influence_rankings;

SELECT NULLIF(reach_count, 0) FROM reach_metrics;

SELECT CASE WHEN engagement_rate > 5 THEN 'high' WHEN engagement_rate > 1 THEN 'medium' ELSE 'low' END FROM engagement_rates;

SELECT IFNULL(conversion_rate, 0) FROM funnel_metrics;

SELECT NULLIF(cart_abandonment_rate, 0) FROM abandonment_metrics;

SELECT COALESCE(checkout_completion, 0) FROM checkout_funnels;

SELECT CASE WHEN upsell_success > 20 THEN 'excellent' WHEN upsell_success > 10 THEN 'good' ELSE 'low' END FROM upsell_metrics;

SELECT NULLIF(crosssell_success, 0) FROM crosssell_analysis;

SELECT COALESCE(addon_adoption, 0) FROM addon_metrics;

SELECT NULLIF(subscription_conversion, 0) FROM subscription_funnels;

SELECT CASE WHEN annual_plan_ratio > 0.5 THEN 'healthy mix' WHEN annual_plan_ratio > 0.3 THEN 'room for improvement' ELSE 'revenue at risk' END FROM plan_mix_analysis;

SELECT IFNULL(mrr, 0) FROM revenue_metrics;

SELECT NULLIF(arr, 0) FROM annual_recurring_revenue;

SELECT COALESCE(expansion_revenue, 0) FROM expansion_metrics;

SELECT NULLIF(churned_mrr, 0) FROM churn_revenue;

SELECT CASE WHEN net_revenue_retention > 100 THEN 'expanding' WHEN net_revenue_retention = 100 THEN 'stable' ELSE 'contracting' END FROM nrr_analysis;

SELECT IFNULL(gross_margin, 0) FROM profitability_analysis;

SELECT NULLIF(net_margin, 0) FROM net_profitability;

SELECT COALESCE(burn_rate, 0) FROM cash_flow_metrics;

SELECT NULLIF(runway_months, 0) FROM runway_calculations;

SELECT CASE WHEN runway_months > 18 THEN 'healthy' WHEN runway_months > 12 THEN 'adequate' ELSE 'concerning' END FROM runway_assessment;
