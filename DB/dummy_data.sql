

SET search_path = public, pg_temp;

-- ===== Users =====
INSERT INTO users (user_id,email,phone,first_name,last_name,citizen_id,password)
VALUES
 ('a39b6135-22bf-4b06-b4b7-e34daa460258','doctor@example.com','0800000001','Alice','Smith','1234567890123','hashed_password1'),
 ('0e01abb3-da7e-4aaf-8b10-4834625f19bb','patient@example.com','0800000002','Bob','Johnson','2345678901234','hashed_password2'),
 ('bc78e582-1318-4fb6-b2a2-38c1d4e74fc0','admin@example.com','0800000003','Charlie','Admin','3456789012345','hashed_password3')
ON CONFLICT (user_id) DO NOTHING;

-- ===== Roles =====
INSERT INTO user_roles (user_id,role) VALUES
 ('a39b6135-22bf-4b06-b4b7-e34daa460258','DOCTOR'),
 ('0e01abb3-da7e-4aaf-8b10-4834625f19bb','PATIENT'),
 ('bc78e582-1318-4fb6-b2a2-38c1d4e74fc0','ADMIN')
ON CONFLICT DO NOTHING;

-- ===== Profiles =====
INSERT INTO doctor_profile (user_id,mln,department,position)
VALUES ('a39b6135-22bf-4b06-b4b7-e34daa460258','MLN123456','Cardiology','Senior Cardiologist')
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO patient_profile (user_id,hn)
VALUES ('0e01abb3-da7e-4aaf-8b10-4834625f19bb',100001)
ON CONFLICT (user_id) DO NOTHING;

-- ===== Time slots (Mon 09–12, Wed 14–17) =====
INSERT INTO time_slots (doctor_id,day_of_weeks,start_time,end_time,place_name)
SELECT 'a39b6135-22bf-4b06-b4b7-e34daa460258',1,'09:00'::time,'12:00'::time,'Clinic 1'
WHERE NOT EXISTS (
  SELECT 1 FROM time_slots
  WHERE doctor_id='a39b6135-22bf-4b06-b4b7-e34daa460258' AND day_of_weeks=1
    AND start_time='09:00' AND end_time='12:00'
);

INSERT INTO time_slots (doctor_id,day_of_weeks,start_time,end_time,place_name)
SELECT 'a39b6135-22bf-4b06-b4b7-e34daa460258',3,'14:00'::time,'17:00'::time,'Clinic 1'
WHERE NOT EXISTS (
  SELECT 1 FROM time_slots
  WHERE doctor_id='a39b6135-22bf-4b06-b4b7-e34daa460258' AND day_of_weeks=3
    AND start_time='14:00' AND end_time='17:00'
);

-- ===== Appointment (Tue 2025-09-23 using the Mon 09:00 slot) =====
INSERT INTO appointments (patient_id,timeslot_id,date,status)
SELECT '0e01abb3-da7e-4aaf-8b10-4834625f19bb', ts.timeslot_id, DATE '2025-09-23', 'ACCEPTED'
FROM time_slots ts
WHERE ts.doctor_id='a39b6135-22bf-4b06-b4b7-e34daa460258' AND ts.day_of_weeks=1 AND ts.start_time='09:00'
ON CONFLICT (patient_id,timeslot_id,date) DO NOTHING;

-- ===== Diagnosis =====
INSERT INTO diagnoses (appointment_id,patient_id,doctor_id,symptom)
SELECT a.appointment_id,
       '0e01abb3-da7e-4aaf-8b10-4834625f19bb',
       'a39b6135-22bf-4b06-b4b7-e34daa460258',
       'Mild fever and sore throat'
FROM appointments a
JOIN time_slots ts ON ts.timeslot_id=a.timeslot_id
WHERE a.patient_id='0e01abb3-da7e-4aaf-8b10-4834625f19bb'
  AND a.date=DATE '2025-09-23'
  AND ts.day_of_weeks=1 AND ts.start_time='09:00'
  AND NOT EXISTS (SELECT 1 FROM diagnoses d WHERE d.appointment_id=a.appointment_id);

-- ===== Medicines (with real image URLs) =====
INSERT INTO medicines (medicine_name,details,unit_price,image_url)
SELECT 'Paracetamol 500 mg tablets','Pain reliever and fever reducer',0.05,
       'https://upload.wikimedia.org/wikipedia/commons/thumb/8/80/Paracetamol_acetaminophen_500_mg_pills.jpg/640px-Paracetamol_acetaminophen_500_mg_pills.jpg'
WHERE NOT EXISTS (SELECT 1 FROM medicines WHERE medicine_name='Paracetamol 500 mg tablets');

INSERT INTO medicines (medicine_name,details,unit_price,image_url)
SELECT 'Ibuprofen 200 mg tablets','Non-steroidal anti-inflammatory drug',0.10,
       'https://upload.wikimedia.org/wikipedia/commons/thumb/9/99/Bottle_of_Ibuprofen_tablets_with_cap_removed_and_tablets_in_front.jpg/500px-Bottle_of_Ibuprofen_tablets_with_cap_removed_and_tablets_in_front.jpg'
WHERE NOT EXISTS (SELECT 1 FROM medicines WHERE medicine_name='Ibuprofen 200 mg tablets');

INSERT INTO medicines (medicine_name,details,unit_price,image_url)
SELECT 'Amoxicillin 500 mg capsules','Penicillin antibiotic for bacterial infections',0.20,
       'https://upload.wikimedia.org/wikipedia/commons/thumb/0/02/Amoxicillin.JPG/621px-Amoxicillin.JPG'
WHERE NOT EXISTS (SELECT 1 FROM medicines WHERE medicine_name='Amoxicillin 500 mg capsules');

-- ===== Prescription (Ibuprofen) =====
INSERT INTO prescriptions (patient_id,medicine_id,dosage,amount,on_going,doctor_comment)
SELECT '0e01abb3-da7e-4aaf-8b10-4834625f19bb', m.medicine_id, '1 tablet three times daily', 30, false, 'Take after meals'
FROM medicines m
WHERE m.medicine_name='Ibuprofen 200 mg tablets'
  AND NOT EXISTS (
    SELECT 1 FROM prescriptions p
    WHERE p.patient_id='0e01abb3-da7e-4aaf-8b10-4834625f19bb' AND p.medicine_id=m.medicine_id
  );

-- ===== Medical rights & assignment =====
INSERT INTO medical_rights (name,details,img_url)
SELECT 'Universal Coverage Scheme','Government-sponsored healthcare coverage',
       'https://upload.wikimedia.org/wikipedia/commons/a/ae/HealthInsuranceCard.jpg'
WHERE NOT EXISTS (SELECT 1 FROM medical_rights WHERE name='Universal Coverage Scheme');

INSERT INTO medical_rights (name,details,img_url)
SELECT 'Social Security Scheme','Employer-based health insurance',
       'https://upload.wikimedia.org/wikipedia/commons/a/ae/HealthInsuranceCard.jpg'
WHERE NOT EXISTS (SELECT 1 FROM medical_rights WHERE name='Social Security Scheme');

INSERT INTO user_mr (patient_id,mr_id)
SELECT '0e01abb3-da7e-4aaf-8b10-4834625f19bb', mr.mr_id
FROM medical_rights mr
WHERE mr.name='Universal Coverage Scheme'
ON CONFLICT DO NOTHING;

-- ===== One “SHIPPING” order with an image =====
INSERT INTO orders (patient_id,status,shipping_platform,payment_platform,image_url)
SELECT '0e01abb3-da7e-4aaf-8b10-4834625f19bb','SHIPPING','DHL','Credit Card',
       'https://upload.wikimedia.org/wikipedia/commons/7/77/Cardboard_box_with_office_supplies.jpg'
WHERE NOT EXISTS (
  SELECT 1 FROM orders WHERE patient_id='0e01abb3-da7e-4aaf-8b10-4834625f19bb' AND status='SHIPPING'
);

-- ===== Order items =====
-- Paracetamol x2
INSERT INTO order_items (order_id,medicine_id,quantity,unit_price)
SELECT ord.order_id, med.medicine_id, 2, med.unit_price
FROM (SELECT order_id
      FROM orders
      WHERE patient_id='0e01abb3-da7e-4aaf-8b10-4834625f19bb' AND status='SHIPPING'
      ORDER BY created_at DESC LIMIT 1) ord,
     (SELECT medicine_id, unit_price
      FROM medicines
      WHERE medicine_name='Paracetamol 500 mg tablets') med
WHERE NOT EXISTS (
  SELECT 1 FROM order_items oi WHERE oi.order_id=ord.order_id AND oi.medicine_id=med.medicine_id
);

-- Ibuprofen x1
INSERT INTO order_items (order_id,medicine_id,quantity,unit_price)
SELECT ord.order_id, med.medicine_id, 1, med.unit_price
FROM (SELECT order_id
      FROM orders
      WHERE patient_id='0e01abb3-da7e-4aaf-8b10-4834625f19bb' AND status='SHIPPING'
      ORDER BY created_at DESC LIMIT 1) ord,
     (SELECT medicine_id, unit_price
      FROM medicines
      WHERE medicine_name='Ibuprofen 200 mg tablets') med
WHERE NOT EXISTS (
  SELECT 1 FROM order_items oi WHERE oi.order_id=ord.order_id AND oi.medicine_id=med.medicine_id
);

-- ===== Shipping address =====
INSERT INTO shipping_address (patient_id,first_name,last_name,address,postal_code,phone,lat,lon)
VALUES ('0e01abb3-da7e-4aaf-8b10-4834625f19bb','Bob','Johnson',
        '123 Sukhumvit Road, Bangkok','10110','0812345678',13.7563,100.5018)
ON CONFLICT (patient_id) DO NOTHING;

-- ===== Shipping status (TIMESTAMPTZ with +07) =====
INSERT INTO shipping_status (order_id,details,lat,lon,at)
SELECT ord.order_id,'Package received at warehouse',13.7000,100.5000,'2025-09-21 10:00:00+07'::timestamptz
FROM (SELECT order_id FROM orders
      WHERE patient_id='0e01abb3-da7e-4aaf-8b10-4834625f19bb' AND status='SHIPPING'
      ORDER BY created_at DESC LIMIT 1) ord
WHERE NOT EXISTS (
  SELECT 1 FROM shipping_status s WHERE s.order_id=ord.order_id AND s.details='Package received at warehouse'
);

INSERT INTO shipping_status (order_id,details,lat,lon,at)
SELECT ord.order_id,'Out for delivery',13.7400,100.5200,'2025-09-22 08:30:00+07'::timestamptz
FROM (SELECT order_id FROM orders
      WHERE patient_id='0e01abb3-da7e-4aaf-8b10-4834625f19bb' AND status='SHIPPING'
      ORDER BY created_at DESC LIMIT 1) ord
WHERE NOT EXISTS (
  SELECT 1 FROM shipping_status s WHERE s.order_id=ord.order_id AND s.details='Out for delivery'
);

INSERT INTO shipping_status (order_id,details,lat,lon,at)
SELECT ord.order_id,'Delivered',13.7563,100.5018,'2025-09-23 15:45:00+07'::timestamptz
FROM (SELECT order_id FROM orders
      WHERE patient_id='0e01abb3-da7e-4aaf-8b10-4834625f19bb' AND status='SHIPPING'
      ORDER BY created_at DESC LIMIT 1) ord
WHERE NOT EXISTS (
  SELECT 1 FROM shipping_status s WHERE s.order_id=ord.order_id AND s.details='Delivered'
);

-- ===== Patient health profile =====
INSERT INTO patient_health_info (patient_id,age,gender,height_cm,weight_kg,medical_conditions,drug_allergies)
VALUES ('0e01abb3-da7e-4aaf-8b10-4834625f19bb',35,'Male',175.0,70.0,'Hypertension','Penicillin')
ON CONFLICT (patient_id) DO NOTHING;
